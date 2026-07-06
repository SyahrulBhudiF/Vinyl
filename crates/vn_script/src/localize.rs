use fluent_syntax::ast::{Entry, PatternElement};
use fluent_syntax::parser;
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

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
