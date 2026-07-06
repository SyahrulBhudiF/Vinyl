use crate::diagnostics::Diagnostic;
use crate::localize::{LocaleCatalog, LocaleError, load_locale};
use crate::manifest::{ManifestError, ProjectManifest, load_manifest};
use crate::parser::{ParseError, parse_file_recovering};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;
use vn_core::Script;

/// Loaded script project.
#[derive(Clone, Debug)]
pub struct LoadedProject {
    pub root: PathBuf,
    pub manifest: ProjectManifest,
    pub script_hash: String,
    pub locales: Vec<LocaleCatalog>,
    pub script: Script,
}

/// Project loading error.
#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("failed to read project directory {path}: {source}")]
    ReadDir { path: PathBuf, source: io::Error },
    #[error("failed to read script file {path}: {source}")]
    ReadFile { path: PathBuf, source: io::Error },
    #[error("manifest load failed: {0}")]
    Manifest(#[from] ManifestError),
    #[error("script parse failed: {0}")]
    Parse(#[from] ParseError),
    #[error("script parse failed")]
    Diagnostics(Vec<Diagnostic>),
    #[error("locale load failed: {0}")]
    Locale(#[from] LocaleError),
}

/// Loads all `.vn` files under `<project>/script` in path order.
pub fn load_project(root: impl AsRef<Path>) -> Result<LoadedProject, ProjectError> {
    let root = root.as_ref().to_path_buf();
    let manifest = load_manifest(&root)?;
    let script_dir = root.join(&manifest.paths.script);
    let mut files = Vec::new();
    collect_vn_files(&script_dir, &mut files)?;
    files.sort();

    let mut statements = Vec::new();
    let mut diagnostics = Vec::new();
    let mut hasher = blake3::Hasher::new();
    for file in files {
        let source = fs::read_to_string(&file).map_err(|source| ProjectError::ReadFile {
            path: file.clone(),
            source,
        })?;
        hash_script_file(&mut hasher, &root, &file, &source);
        let (parsed, mut parse_diagnostics) = parse_file_recovering(&file, &source);
        statements.extend(parsed.statements);
        diagnostics.append(&mut parse_diagnostics);
    }
    if !diagnostics.is_empty() {
        return Err(ProjectError::Diagnostics(diagnostics));
    }

    let locale_root = root.join(&manifest.paths.locales);
    let locales = load_locales(&locale_root)?;

    Ok(LoadedProject {
        root,
        manifest,
        script_hash: hasher.finalize().to_hex().to_string(),
        locales,
        script: Script { statements },
    })
}

fn load_locales(root: &Path) -> Result<Vec<LocaleCatalog>, ProjectError> {
    let Ok(entries) = fs::read_dir(root) else {
        return Ok(Vec::new());
    };
    let mut locales = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| ProjectError::ReadDir {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.extension().is_some_and(|extension| extension == "ftl")
            && let Some(locale) = path.file_stem().and_then(|stem| stem.to_str())
        {
            locales.push(load_locale(root, locale)?);
        }
    }
    locales.sort_by(|left, right| left.locale.cmp(&right.locale));
    Ok(locales)
}

fn hash_script_file(hasher: &mut blake3::Hasher, root: &Path, file: &Path, source: &str) {
    let relative = file.strip_prefix(root).unwrap_or(file);
    hasher.update(relative.to_string_lossy().as_bytes());
    hasher.update(&[0]);
    hasher.update(source.as_bytes());
    hasher.update(&[0]);
}

fn collect_vn_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), ProjectError> {
    let entries = fs::read_dir(dir).map_err(|source| ProjectError::ReadDir {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| ProjectError::ReadDir {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_vn_files(&path, files)?;
        } else if path.extension().is_some_and(|extension| extension == "vn") {
            files.push(path);
        }
    }
    Ok(())
}
