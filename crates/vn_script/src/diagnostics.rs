use vn_core::SourcePos;

/// User-facing diagnostic with source location.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub pos: SourcePos,
    pub message: String,
}

impl Diagnostic {
    pub fn new(pos: SourcePos, message: impl Into<String>) -> Self {
        Self {
            pos,
            message: message.into(),
        }
    }

    pub fn render(&self) -> String {
        format!(
            "{}:{}:{}: {}",
            self.pos.file, self.pos.line, self.pos.column, self.message
        )
    }
}

/// Collection of diagnostics.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiagnosticSet {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticSet {
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn into_vec(self) -> Vec<Diagnostic> {
        self.diagnostics
    }
}
