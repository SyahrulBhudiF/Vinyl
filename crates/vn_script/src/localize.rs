use fluent_syntax::ast::{Entry, PatternElement};
use fluent_syntax::parser;
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;
use vn_core::{Choice, Script, Stmt, StmtKind};

/// Loaded Fluent messages for one locale.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LocaleCatalog {
    pub locale: String,
    pub messages: BTreeMap<String, String>,
}

impl LocaleCatalog {
    /// Returns the translated message for `id`.
    pub fn get(&self, id: &str) -> Option<&str> {
        self.messages.get(id).map(String::as_str)
    }
}

/// Locale loading error.
#[derive(Debug, Error)]
pub enum LocaleError {
    #[error("failed to read locale {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("failed to parse locale {path}")]
    Parse { path: PathBuf },
}

/// Loads `<locale_root>/<locale>.ftl`.
pub fn load_locale(root: &Path, locale: &str) -> Result<LocaleCatalog, LocaleError> {
    let path = root.join(format!("{locale}.ftl"));
    let source = fs::read_to_string(&path).map_err(|source| LocaleError::Read {
        path: path.clone(),
        source,
    })?;
    parse_locale(&path, locale, &source)
}

/// Extracts text IDs and fallback source text from a script.
pub fn extract_messages(script: &Script) -> BTreeMap<String, String> {
    let mut messages = BTreeMap::new();
    collect_messages(&script.statements, &mut messages);
    messages
}

/// Renders extracted messages as simple Fluent entries.
pub fn render_messages(messages: &BTreeMap<String, String>) -> String {
    messages
        .iter()
        .map(|(id, text)| format!("{id} = {}\n", text.replace('\n', "\\n")))
        .collect()
}

/// Parses one Fluent source into a flat text catalog.
pub fn parse_locale(path: &Path, locale: &str, source: &str) -> Result<LocaleCatalog, LocaleError> {
    let resource = parser::parse(source).map_err(|_| LocaleError::Parse {
        path: path.to_path_buf(),
    })?;
    let mut messages = BTreeMap::new();
    for entry in resource.body {
        let Entry::Message(message) = entry else {
            continue;
        };
        let Some(value) = message.value else {
            continue;
        };
        messages.insert(
            message.id.name.to_string(),
            pattern_to_string(value.elements),
        );
    }
    Ok(LocaleCatalog {
        locale: locale.to_string(),
        messages,
    })
}

fn collect_messages(statements: &[Stmt], messages: &mut BTreeMap<String, String>) {
    for statement in statements {
        match &statement.kind {
            StmtKind::Say {
                text_id: Some(text_id),
                text,
                ..
            } => {
                messages
                    .entry(text_id.clone())
                    .or_insert_with(|| text.clone());
            }
            StmtKind::Menu { choices } => collect_choice_messages(choices, messages),
            StmtKind::If {
                then_body,
                else_body,
                ..
            } => {
                collect_messages(then_body, messages);
                collect_messages(else_body, messages);
            }
            _ => {}
        }
    }
}

fn collect_choice_messages(choices: &[Choice], messages: &mut BTreeMap<String, String>) {
    for choice in choices {
        if let Some(text_id) = &choice.text_id {
            messages
                .entry(text_id.clone())
                .or_insert_with(|| choice.text.clone());
        }
        collect_messages(&choice.body, messages);
    }
}

fn pattern_to_string(elements: Vec<PatternElement<&str>>) -> String {
    elements
        .into_iter()
        .filter_map(|element| match element {
            PatternElement::TextElement { value } => Some(value),
            PatternElement::Placeable { .. } => None,
        })
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_string()
}
