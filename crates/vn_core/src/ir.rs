use crate::ast::{AssignOp, Expr, SourcePos, TextEffect, Transition};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Stable identifier for a compiled operation.
pub type OpId = usize;

/// Compiled runtime program.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub ops: Vec<Op>,
    pub labels: HashMap<String, OpId>,
}

/// Runtime operation with source position.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Op {
    pub kind: OpKind,
    pub pos: SourcePos,
}

/// Deterministic VM bytecode.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum OpKind {
    Say {
        speaker: Option<String>,
        text_id: Option<String>,
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
        choices: Vec<MenuChoice>,
    },
    Jump {
        target: OpId,
    },
    Set {
        var: String,
        op: AssignOp,
        value: Expr,
    },
    Branch {
        cond: Expr,
        then_pc: OpId,
        else_pc: OpId,
    },
    End,
}

/// Compiled menu choice.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MenuChoice {
    pub text_id: Option<String>,
    pub text: String,
    pub condition: Option<Expr>,
    pub target: OpId,
}
