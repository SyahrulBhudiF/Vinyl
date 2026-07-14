use crate::asset::{AssetId, AssetResolver};
use crate::diagnostics::{Diagnostic, DiagnosticSet};
use crate::localize::LocaleCatalog;
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
    validate_with_locales(script, project_root, manifest, &[])
}

/// Validates labels, assets, and locale entries using explicit locale catalogs.
pub fn validate_with_locales(
    script: &Script,
    project_root: &Path,
    manifest: &ProjectManifest,
    locales: &[LocaleCatalog],
) -> Result<(), ValidationError> {
    let mut context = ValidationContext::new(project_root, manifest.clone(), locales);
    context.collect_labels(&script.statements);
    context.require_start_label(script);
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
    text_ids: HashSet<String>,
    locales: Vec<LocaleCatalog>,
    diagnostics: DiagnosticSet,
}

impl ValidationContext {
    fn new(project_root: &Path, manifest: ProjectManifest, locales: &[LocaleCatalog]) -> Self {
        Self {
            resolver: AssetResolver::new(project_root, manifest),
            labels: HashSet::new(),
            text_ids: HashSet::new(),
            locales: locales.to_vec(),
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

    fn require_start_label(&mut self, script: &Script) {
        if !self.labels.contains("start") {
            let pos = script
                .statements
                .first()
                .map(|statement| statement.pos.clone())
                .unwrap_or_else(|| SourcePos {
                    file: "<project>".to_string(),
                    line: 1,
                    column: 1,
                });
            self.diagnostics
                .push(Diagnostic::new(pos, "missing required label 'start'"));
        }
    }

    fn validate_statements(&mut self, statements: &[Stmt]) {
        for statement in statements {
            match &statement.kind {
                StmtKind::Scene { image, .. } => {
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
                StmtKind::Say {
                    text_id: Some(text_id),
                    ..
                } => self.validate_text_id(statement.pos.clone(), text_id),
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
            if let Some(text_id) = &choice.text_id {
                self.validate_text_id(choice.pos.clone(), text_id);
            }
            self.validate_statements(&choice.body);
        }
    }

    fn validate_text_id(&mut self, pos: SourcePos, text_id: &str) {
        if !self.text_ids.insert(text_id.to_string()) {
            self.diagnostics.push(Diagnostic::new(
                pos.clone(),
                format!("duplicate text id '{text_id}'"),
            ));
        }
        for locale in &self.locales {
            if locale.get(text_id).is_none() {
                self.diagnostics.push(Diagnostic::new(
                    pos.clone(),
                    format!("missing locale '{}' entry '{text_id}'", locale.locale),
                ));
            }
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
