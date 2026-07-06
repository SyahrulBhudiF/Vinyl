//! Parser and validator for VN script projects.

pub mod asset;
pub mod diagnostics;
pub mod localize;
pub mod manifest;
pub mod parser;
pub mod project;
pub mod validate;

pub use asset::{AssetId, AssetResolver};
pub use diagnostics::{Diagnostic, DiagnosticSet};
pub use localize::{LocaleCatalog, LocaleError, load_locale, parse_locale};
pub use manifest::{AssetPaths, ManifestError, ProjectManifest, ProjectMetadata, ProjectPaths};
pub use parser::{ParseError, parse_file, parse_source};
pub use project::{LoadedProject, ProjectError, load_project};
pub use validate::{ValidationError, validate, validate_with_locales, validate_with_manifest};
