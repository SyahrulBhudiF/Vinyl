//! Renderer-independent Vinyl core.

pub mod ast;
pub mod compile;
pub mod ir;
pub mod save;
pub mod vm;

pub use ast::{
    AssignOp, BinaryOp, Choice, Expr, Script, SourcePos, Stmt, StmtKind, TextEffect, Transition,
    UnaryOp, Value,
};
pub use compile::compile;
pub use ir::{MenuChoice, Op, OpId, OpKind, Program};
pub use save::{
    CURRENT_SAVE_VERSION, DialogueSnapshot, Preferences, PresentationSnapshot, ProjectId,
    RollbackCheckpoint, SaveFile, SaveValidationError, SpriteSnapshot, validate_save,
};
pub use vm::{HistoryEntry, Vm, VmError, VmEvent, VmState};
