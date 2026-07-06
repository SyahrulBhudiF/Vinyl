use crate::ast::{AssignOp, BinaryOp, Expr, TextEffect, Transition, UnaryOp, Value};
use crate::ir::{MenuChoice, OpId, OpKind, Program};
use crate::save::{DialogueSnapshot, PresentationSnapshot, SpriteSnapshot};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Serializable VM state.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct VmState {
    pub pc: OpId,
    pub variables: HashMap<String, Value>,
    pub current_choices: Vec<MenuChoice>,
    pub history: Vec<HistoryEntry>,
}

/// Story history entry for rollback/debug UI.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub speaker: Option<String>,
    pub text: String,
}

/// VM events consumed by UI/presentation orchestration.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum VmEvent {
    Dialogue {
        speaker: Option<String>,
        text: String,
        effect: TextEffect,
    },
    Scene {
        image: String,
        transition: Option<Transition>,
    },
    Show {
        tag: String,
        attrs: Vec<String>,
        position: String,
        transition: Option<Transition>,
    },
    Hide {
        tag: String,
    },
    PlayMusic {
        path: String,
    },
    StopMusic,
    Menu {
        choices: Vec<String>,
    },
    End,
}

/// VM execution error.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum VmError {
    #[error("program counter {pc} is outside program")]
    InvalidProgramCounter { pc: OpId },
    #[error("choice {choice} is outside active menu")]
    InvalidChoice { choice: usize },
    #[error("expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
    #[error("unknown variable {name}")]
    UnknownVariable { name: String },
}

/// Deterministic story virtual machine.
pub struct Vm {
    program: Program,
    state: VmState,
    rollback: Vec<(VmState, PresentationSnapshot)>,
    presentation: PresentationSnapshot,
}

impl Vm {
    /// Creates a VM at program start.
    pub fn new(program: Program) -> Self {
        Self {
            program,
            state: VmState::default(),
            rollback: Vec::new(),
            presentation: PresentationSnapshot::default(),
        }
    }

    /// Restores a VM from serialized state and presentation.
    pub fn from_parts(
        program: Program,
        state: VmState,
        presentation: PresentationSnapshot,
    ) -> Self {
        Self {
            program,
            state,
            rollback: Vec::new(),
            presentation,
        }
    }

    /// Returns current VM state.
    pub fn state(&self) -> &VmState {
        &self.state
    }

    /// Returns current presentation snapshot.
    pub fn presentation(&self) -> &PresentationSnapshot {
        &self.presentation
    }

    /// Advances until dialogue, menu, visual/audio event, or end.
    pub fn continue_until_interaction(&mut self) -> Result<VmEvent, VmError> {
        loop {
            let kind = self
                .program
                .ops
                .get(self.state.pc)
                .ok_or(VmError::InvalidProgramCounter { pc: self.state.pc })?
                .kind
                .clone();
            match kind {
                OpKind::Say {
                    speaker,
                    text,
                    effect,
                } => {
                    self.checkpoint();
                    self.state.pc += 1;
                    self.state.history.push(HistoryEntry {
                        speaker: speaker.clone(),
                        text: text.clone(),
                    });
                    self.presentation.dialogue = Some(DialogueSnapshot {
                        speaker: speaker.clone(),
                        text: text.clone(),
                    });
                    self.presentation.menu = None;
                    return Ok(VmEvent::Dialogue {
                        speaker: speaker.clone(),
                        text: text.clone(),
                        effect,
                    });
                }
                OpKind::Scene { image, transition } => {
                    self.state.pc += 1;
                    self.presentation.background = Some(image.clone());
                    self.presentation.sprites.clear();
                    return Ok(VmEvent::Scene {
                        image: image.clone(),
                        transition,
                    });
                }
                OpKind::Show {
                    tag,
                    attrs,
                    position,
                    transition,
                } => {
                    self.state.pc += 1;
                    self.presentation.sprites.insert(
                        tag.clone(),
                        SpriteSnapshot {
                            attrs: attrs.clone(),
                            position: position.clone(),
                        },
                    );
                    return Ok(VmEvent::Show {
                        tag: tag.clone(),
                        attrs: attrs.clone(),
                        position: position.clone(),
                        transition,
                    });
                }
                OpKind::Hide { tag } => {
                    self.state.pc += 1;
                    self.presentation.sprites.remove(&tag);
                    return Ok(VmEvent::Hide { tag });
                }
                OpKind::PlayMusic { path } => {
                    self.state.pc += 1;
                    self.presentation.music = Some(path.clone());
                    return Ok(VmEvent::PlayMusic { path: path.clone() });
                }
                OpKind::StopMusic => {
                    self.state.pc += 1;
                    self.presentation.music = None;
                    return Ok(VmEvent::StopMusic);
                }
                OpKind::Menu { choices } => {
                    self.checkpoint();
                    self.state.current_choices = self.visible_choices(&choices)?;
                    let texts = self
                        .state
                        .current_choices
                        .iter()
                        .map(|choice| choice.text.clone())
                        .collect();
                    self.presentation.menu = Some(texts);
                    return Ok(VmEvent::Menu {
                        choices: self.presentation.menu.clone().unwrap_or_default(),
                    });
                }
                OpKind::Jump { target } => self.state.pc = target,
                OpKind::Set { var, op, value } => {
                    let value = self.eval_assign(&var, op, &value)?;
                    self.state.variables.insert(var, value);
                    self.state.pc += 1;
                }
                OpKind::Branch {
                    cond,
                    then_pc,
                    else_pc,
                } => {
                    self.state.pc = if self.eval_bool(&cond)? {
                        then_pc
                    } else {
                        else_pc
                    };
                }
                OpKind::End => {
                    self.state.current_choices.clear();
                    self.presentation.menu = None;
                    return Ok(VmEvent::End);
                }
            }
        }
    }

    /// Chooses an active menu branch, then advances to the next interaction.
    pub fn choose(&mut self, choice: usize) -> Result<VmEvent, VmError> {
        let rollback_state = self.state.clone();
        let rollback_presentation = self.presentation.clone();
        let target = self
            .state
            .current_choices
            .get(choice)
            .ok_or(VmError::InvalidChoice { choice })?
            .target;
        self.state.current_choices.clear();
        self.presentation.menu = None;
        self.state.pc = target;
        let event = self.continue_until_interaction()?;
        let _discard_choice_body_checkpoint = self.rollback.pop();
        self.rollback.push((rollback_state, rollback_presentation));
        Ok(event)
    }

    /// Restores the previous interaction checkpoint.
    pub fn rollback(&mut self) -> Option<VmEvent> {
        let (state, presentation) = self.rollback.pop()?;
        self.state = state;
        self.presentation = presentation;
        if let Some(choices) = &self.presentation.menu {
            return Some(VmEvent::Menu {
                choices: choices.clone(),
            });
        }
        if let Some(dialogue) = &self.presentation.dialogue {
            return Some(VmEvent::Dialogue {
                speaker: dialogue.speaker.clone(),
                text: dialogue.text.clone(),
                effect: TextEffect::Instant,
            });
        }
        None
    }

    fn checkpoint(&mut self) {
        self.rollback
            .push((self.state.clone(), self.presentation.clone()));
    }

    fn visible_choices(&self, choices: &[MenuChoice]) -> Result<Vec<MenuChoice>, VmError> {
        let mut visible = Vec::new();
        for choice in choices {
            let keep = match &choice.condition {
                Some(condition) => self.eval_bool(condition)?,
                None => true,
            };
            if keep {
                visible.push(choice.clone());
            }
        }
        Ok(visible)
    }

    fn eval_assign(&self, var: &str, op: AssignOp, value: &Expr) -> Result<Value, VmError> {
        let value = self.eval_value(value)?;
        match op {
            AssignOp::Set => Ok(value),
            AssignOp::Add => Ok(Value::Int(self.variable_int(var)? + value.as_int()?)),
            AssignOp::Sub => Ok(Value::Int(self.variable_int(var)? - value.as_int()?)),
        }
    }

    fn eval_bool(&self, expr: &Expr) -> Result<bool, VmError> {
        self.eval_value(expr)?.as_bool()
    }

    fn eval_value(&self, expr: &Expr) -> Result<Value, VmError> {
        match expr {
            Expr::Value(value) => Ok(value.clone()),
            Expr::Var(name) => self
                .state
                .variables
                .get(name)
                .cloned()
                .ok_or_else(|| VmError::UnknownVariable { name: name.clone() }),
            Expr::Unary { op, expr } => match op {
                UnaryOp::Not => Ok(Value::Bool(!self.eval_value(expr)?.as_bool()?)),
            },
            Expr::Binary { left, op, right } => self.eval_binary(left, op, right),
        }
    }

    fn eval_binary(&self, left: &Expr, op: &BinaryOp, right: &Expr) -> Result<Value, VmError> {
        match op {
            BinaryOp::Or => Ok(Value::Bool(self.eval_bool(left)? || self.eval_bool(right)?)),
            BinaryOp::And => Ok(Value::Bool(self.eval_bool(left)? && self.eval_bool(right)?)),
            BinaryOp::Eq => Ok(Value::Bool(
                self.eval_value(left)? == self.eval_value(right)?,
            )),
            BinaryOp::Ne => Ok(Value::Bool(
                self.eval_value(left)? != self.eval_value(right)?,
            )),
            BinaryOp::Lt => Ok(Value::Bool(
                self.eval_value(left)?.as_int()? < self.eval_value(right)?.as_int()?,
            )),
            BinaryOp::Le => Ok(Value::Bool(
                self.eval_value(left)?.as_int()? <= self.eval_value(right)?.as_int()?,
            )),
            BinaryOp::Gt => Ok(Value::Bool(
                self.eval_value(left)?.as_int()? > self.eval_value(right)?.as_int()?,
            )),
            BinaryOp::Ge => Ok(Value::Bool(
                self.eval_value(left)?.as_int()? >= self.eval_value(right)?.as_int()?,
            )),
            BinaryOp::Add => Ok(Value::Int(
                self.eval_value(left)?.as_int()? + self.eval_value(right)?.as_int()?,
            )),
            BinaryOp::Sub => Ok(Value::Int(
                self.eval_value(left)?.as_int()? - self.eval_value(right)?.as_int()?,
            )),
        }
    }

    fn variable_int(&self, name: &str) -> Result<i64, VmError> {
        self.state
            .variables
            .get(name)
            .ok_or_else(|| VmError::UnknownVariable {
                name: name.to_string(),
            })?
            .as_int()
    }
}

impl Value {
    fn as_bool(&self) -> Result<bool, VmError> {
        match self {
            Self::Bool(value) => Ok(*value),
            other => Err(VmError::TypeMismatch {
                expected: "bool".to_string(),
                actual: other.type_name().to_string(),
            }),
        }
    }

    fn as_int(&self) -> Result<i64, VmError> {
        match self {
            Self::Int(value) => Ok(*value),
            other => Err(VmError::TypeMismatch {
                expected: "int".to_string(),
                actual: other.type_name().to_string(),
            }),
        }
    }

    fn type_name(&self) -> &'static str {
        match self {
            Self::Bool(_) => "bool",
            Self::Int(_) => "int",
            Self::Str(_) => "string",
        }
    }
}
