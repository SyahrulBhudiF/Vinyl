use crate::asset::{AssetId, AssetResolver};
use crate::diagnostics::{Diagnostic, DiagnosticSet};
use crate::manifest::ProjectManifest;
use std::collections::HashSet;
use std::path::Path;
use thiserror::Error;
use vn_core::{Choice, Script, SourcePos, Stmt, StmtKind};

/// Validation failure with one or more diagnostics.
#[derive(Debug, Error)]
#[error("validation failed")]
pub struct ValidationError {
    diagnostics: Vec<Diagnostic>,
}

impl ValidationError {
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

/// Validates labels and assets for a loaded project.
pub fn validate(script: &Script, project_root: &Path) -> Result<(), ValidationError> {
    validate_with_manifest(
        script,
        project_root,
        &ProjectManifest::default_for_root(project_root),
    )
}

/// Validates labels and assets using an explicit manifest.
pub fn validate_with_manifest(
    script: &Script,
    project_root: &Path,
    manifest: &ProjectManifest,
) -> Result<(), ValidationError> {
    let mut context = ValidationContext::new(project_root, manifest.clone());
    context.collect_labels(&script.statements);
    context.validate_statements(&script.statements);
    if context.diagnostics.is_empty() {
        Ok(())
    } else {
        Err(ValidationError {
            diagnostics: context.diagnostics.into_vec(),
        })
    }
}

struct ValidationContext {
    resolver: AssetResolver,
    labels: HashSet<String>,
    diagnostics: DiagnosticSet,
}

impl ValidationContext {
    fn new(project_root: &Path, manifest: ProjectManifest) -> Self {
        Self {
            resolver: AssetResolver::new(project_root, manifest),
            labels: HashSet::new(),
            diagnostics: DiagnosticSet::default(),
        }
    }

    fn collect_labels(&mut self, statements: &[Stmt]) {
        for statement in statements {
            match &statement.kind {
                StmtKind::Label { name } => {
                    if !self.labels.insert(name.clone()) {
                        self.diagnostics.push(Diagnostic::new(
                            statement.pos.clone(),
                            format!("duplicate label '{name}'"),
                        ));
                    }
                }
                StmtKind::Menu { choices } => self.collect_choice_labels(choices),
                StmtKind::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    self.collect_labels(then_body);
                    self.collect_labels(else_body);
                }
                _ => {}
            }
        }
    }

    fn collect_choice_labels(&mut self, choices: &[Choice]) {
        for choice in choices {
            self.collect_labels(&choice.body);
        }
    }

    fn validate_statements(&mut self, statements: &[Stmt]) {
        for statement in statements {
            match &statement.kind {
                StmtKind::Scene { image } => {
                    self.validate_asset(statement.pos.clone(), AssetId::Background(image.clone()))
                }
                StmtKind::Show { tag, attrs, .. } => self.validate_asset(
                    statement.pos.clone(),
                    AssetId::Sprite {
                        tag: tag.clone(),
                        attrs: attrs.clone(),
                    },
                ),
                StmtKind::PlayMusic { path } => {
                    self.validate_asset(statement.pos.clone(), AssetId::Audio(path.clone()));
                }
                StmtKind::Jump { label } if !self.labels.contains(label) => self.diagnostics.push(
                    Diagnostic::new(statement.pos.clone(), format!("missing label '{label}'")),
                ),
                StmtKind::Menu { choices } => self.validate_choices(choices),
                StmtKind::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    self.validate_statements(then_body);
                    self.validate_statements(else_body);
                }
                _ => {}
            }
        }
    }

    fn validate_choices(&mut self, choices: &[Choice]) {
        for choice in choices {
            self.validate_statements(&choice.body);
        }
    }

    fn validate_asset(&mut self, pos: SourcePos, asset: AssetId) {
        let path = self.resolver.resolve(&asset);
        if !path.exists() {
            self.diagnostics.push(Diagnostic::new(
                pos,
                format!("missing asset '{}'", path.display()),
            ));
        }
    }
}
