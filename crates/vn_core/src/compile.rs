use crate::ast::{Choice, Expr, Script, Stmt, StmtKind};
use crate::ir::{MenuChoice, Op, OpId, OpKind, Program};
use std::collections::HashMap;

/// Compiles a parsed script into runtime bytecode.
pub fn compile(script: &Script) -> Program {
    let mut compiler = Compiler::default();
    compiler.compile_statements(&script.statements);
    compiler.patch_jumps();
    Program {
        ops: compiler.ops,
        labels: compiler.labels,
    }
}

#[derive(Default)]
struct Compiler {
    ops: Vec<Op>,
    labels: HashMap<String, OpId>,
    pending_jumps: Vec<(OpId, String)>,
}

impl Compiler {
    fn compile_statements(&mut self, statements: &[Stmt]) {
        for statement in statements {
            match &statement.kind {
                StmtKind::Label { name } => {
                    self.labels.insert(name.clone(), self.ops.len());
                }
                StmtKind::Say {
                    speaker,
                    text_id,
                    text,
                    effect,
                } => self.push(
                    OpKind::Say {
                        speaker: speaker.clone(),
                        text_id: text_id.clone(),
                        text: text.clone(),
                        effect: effect.clone(),
                    },
                    statement,
                ),
                StmtKind::Scene { image, transition } => {
                    self.push(
                        OpKind::Scene {
                            image: image.clone(),
                            transition: transition.clone(),
                        },
                        statement,
                    );
                }
                StmtKind::Show {
                    tag,
                    attrs,
                    position,
                    transition,
                } => self.push(
                    OpKind::Show {
                        tag: tag.clone(),
                        attrs: attrs.clone(),
                        position: position.clone(),
                        transition: transition.clone(),
                    },
                    statement,
                ),
                StmtKind::Hide { tag } => self.push(OpKind::Hide { tag: tag.clone() }, statement),
                StmtKind::PlayMusic { path } => {
                    self.push(OpKind::PlayMusic { path: path.clone() }, statement);
                }
                StmtKind::StopMusic => self.push(OpKind::StopMusic, statement),
                StmtKind::Menu { choices } => self.compile_menu(choices, statement),
                StmtKind::Jump { label } => {
                    let pc = self.ops.len();
                    self.pending_jumps.push((pc, label.clone()));
                    self.push(OpKind::Jump { target: 0 }, statement);
                }
                StmtKind::Set { var, op, value } => self.push(
                    OpKind::Set {
                        var: var.clone(),
                        op: op.clone(),
                        value: value.clone(),
                    },
                    statement,
                ),
                StmtKind::If {
                    cond,
                    then_body,
                    else_body,
                } => self.compile_if(cond.clone(), then_body, else_body, statement),
                StmtKind::End => self.push(OpKind::End, statement),
            }
        }
    }

    fn compile_menu(&mut self, choices: &[Choice], statement: &Stmt) {
        let menu_pc = self.ops.len();
        self.push(
            OpKind::Menu {
                choices: Vec::new(),
            },
            statement,
        );
        let mut compiled_choices = Vec::with_capacity(choices.len());
        let mut end_jumps = Vec::with_capacity(choices.len());
        for choice in choices {
            let target = self.ops.len();
            compiled_choices.push(MenuChoice {
                text_id: choice.text_id.clone(),
                text: choice.text.clone(),
                condition: choice.condition.clone(),
                target,
            });
            self.compile_statements(&choice.body);
            let jump_pc = self.ops.len();
            end_jumps.push(jump_pc);
            self.push(OpKind::Jump { target: 0 }, statement);
        }
        let end_pc = self.ops.len();
        if let OpKind::Menu { choices } = &mut self.ops[menu_pc].kind {
            *choices = compiled_choices;
        }
        for jump_pc in end_jumps {
            self.ops[jump_pc].kind = OpKind::Jump { target: end_pc };
        }
    }

    fn compile_if(&mut self, cond: Expr, then_body: &[Stmt], else_body: &[Stmt], statement: &Stmt) {
        let branch_pc = self.ops.len();
        self.push(
            OpKind::Branch {
                cond,
                then_pc: 0,
                else_pc: 0,
            },
            statement,
        );
        let then_pc = self.ops.len();
        self.compile_statements(then_body);
        let jump_pc = self.ops.len();
        self.push(OpKind::Jump { target: 0 }, statement);
        let else_pc = self.ops.len();
        self.compile_statements(else_body);
        let end_pc = self.ops.len();
        self.ops[jump_pc].kind = OpKind::Jump { target: end_pc };
        if let OpKind::Branch {
            then_pc: t,
            else_pc: e,
            ..
        } = &mut self.ops[branch_pc].kind
        {
            *t = then_pc;
            *e = else_pc;
        }
    }

    fn push(&mut self, kind: OpKind, statement: &Stmt) {
        self.ops.push(Op {
            kind,
            pos: statement.pos.clone(),
        });
    }

    fn patch_jumps(&mut self) {
        for (pc, label) in &self.pending_jumps {
            if let Some(target) = self.labels.get(label) {
                self.ops[*pc].kind = OpKind::Jump { target: *target };
            }
        }
    }
}
