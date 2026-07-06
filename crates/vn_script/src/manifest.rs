use serde::Deserialize;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Project metadata and content roots loaded from `vinyl.toml`.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct ProjectManifest {
    #[serde(default)]
    pub project: ProjectMetadata,
    #[serde(default)]
    pub paths: ProjectPaths,
    #[serde(default)]
    pub assets: AssetPaths,
}

impl ProjectManifest {
    /// Returns defaults using the project directory name as project id/title.
    pub fn default_for_root(root: &Path) -> Self {
        let name = root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("vinyl")
            .to_string();
        Self {
            project: ProjectMetadata {
                id: name.clone(),
                title: name,
                version: "0.1.0".to_string(),
                default_locale: "en-US".to_string(),
            },
            paths: ProjectPaths::default(),
            assets: AssetPaths::default(),
        }
    }
}

/// Human and compatibility metadata for a project.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct ProjectMetadata {
    pub id: String,
    pub title: String,
    pub version: String,
    pub default_locale: String,
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        Self {
            id: "vinyl".to_string(),
            title: "Vinyl Project".to_string(),
            version: "0.1.0".to_string(),
            default_locale: "en-US".to_string(),
        }
    }
}

/// Project content roots, relative to the project directory.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct ProjectPaths {
    pub script: PathBuf,
    pub assets: PathBuf,
    pub locales: PathBuf,
}

impl Default for ProjectPaths {
    fn default() -> Self {
        Self {
            script: PathBuf::from("script"),
            assets: PathBuf::from("assets"),
            locales: PathBuf::from("locale"),
        }
    }
}

/// Asset category roots, relative to `paths.assets`.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct AssetPaths {
    pub backgrounds: PathBuf,
    pub sprites: PathBuf,
    pub audio: PathBuf,
}

impl Default for AssetPaths {
    fn default() -> Self {
        Self {
            backgrounds: PathBuf::from("bg"),
            sprites: PathBuf::from("sprites"),
            audio: PathBuf::from("audio"),
        }
    }
}

/// Manifest loading error.
#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("failed to read manifest {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("failed to parse manifest {path}: {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
}

/// Loads `<root>/vinyl.toml`, or returns sane defaults when absent.
pub fn load_manifest(root: &Path) -> Result<ProjectManifest, ManifestError> {
    let path = root.join("vinyl.toml");
    match fs::read_to_string(&path) {
        Ok(source) => {
            toml::from_str(&source).map_err(|source| ManifestError::Parse { path, source })
        }
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            Ok(ProjectManifest::default_for_root(root))
        }
        Err(source) => Err(ManifestError::Read { path, source }),
    }
}
