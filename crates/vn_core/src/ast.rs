use serde::{Deserialize, Serialize};

/// Source position carried through diagnostics.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourcePos {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

/// A complete parsed visual novel script.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Script {
    pub statements: Vec<Stmt>,
}

/// High-level authoring AST for the MVP script language.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Stmt {
    pub kind: StmtKind,
    pub pos: SourcePos,
}

/// MVP script statements.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum StmtKind {
    Label {
        name: String,
    },
    Say {
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
        choices: Vec<Choice>,
    },
    Jump {
        label: String,
    },
    Set {
        var: String,
        op: AssignOp,
        value: Expr,
    },
    If {
        cond: Expr,
        then_body: Vec<Stmt>,
        else_body: Vec<Stmt>,
    },
    End,
}

/// Visual transition metadata.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    pub kind: String,
    pub duration_ms: u32,
}

/// Text reveal effect metadata.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum TextEffect {
    #[default]
    Instant,
    Typewriter {
        chars_per_second: u16,
    },
}

/// A menu branch.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Choice {
    pub text: String,
    pub condition: Option<Expr>,
    pub body: Vec<Stmt>,
    pub pos: SourcePos,
}

/// Assignment operator.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AssignOp {
    Set,
    Add,
    Sub,
}

/// Deterministic expression subset.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Value(Value),
    Var(String),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
}

/// Unary expression operator.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum UnaryOp {
    Not,
}

/// Binary expression operator.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum BinaryOp {
    Or,
    And,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Add,
    Sub,
}

/// Serializable script value.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Str(String),
}
