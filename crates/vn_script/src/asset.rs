use crate::manifest::ProjectManifest;
use std::path::{Path, PathBuf};

/// Script-level asset reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssetId {
    Background(String),
    Sprite { tag: String, attrs: Vec<String> },
    Audio(String),
}

/// Resolves script asset references using a loaded project manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetResolver {
    root: PathBuf,
    manifest: ProjectManifest,
}

impl AssetResolver {
    /// Creates a project-rooted resolver.
    pub fn new(root: impl Into<PathBuf>, manifest: ProjectManifest) -> Self {
        Self {
            root: root.into(),
            manifest,
        }
    }

    /// Resolves an asset to an absolute project path.
    pub fn resolve(&self, asset: &AssetId) -> PathBuf {
        match asset {
            AssetId::Background(image) => self.background(image),
            AssetId::Sprite { tag, attrs } => self.sprite(tag, attrs),
            AssetId::Audio(path) => self.audio(path),
        }
    }

    /// Resolves `scene bg room` style identifiers.
    pub fn background(&self, image: &str) -> PathBuf {
        let name = image.strip_prefix("bg ").unwrap_or(image).replace(' ', "/");
        self.asset_root()
            .join(&self.manifest.assets.backgrounds)
            .join(format!("{name}.png"))
    }

    /// Resolves `show eileen happy` style identifiers.
    pub fn sprite(&self, tag: &str, attrs: &[String]) -> PathBuf {
        let file = if attrs.is_empty() {
            "default".to_string()
        } else {
            attrs.join("_")
        };
        self.asset_root()
            .join(&self.manifest.assets.sprites)
            .join(tag)
            .join(format!("{file}.png"))
    }

    /// Resolves audio paths relative to the manifest audio root.
    pub fn audio(&self, path: &str) -> PathBuf {
        let relative = path
            .strip_prefix("assets/audio/")
            .or_else(|| path.strip_prefix("audio/"))
            .unwrap_or(path);
        self.asset_root()
            .join(&self.manifest.assets.audio)
            .join(relative)
    }

    fn asset_root(&self) -> PathBuf {
        self.root.join(&self.manifest.paths.assets)
    }

    /// Returns the resolver root.
    pub fn root(&self) -> &Path {
        &self.root
    }
}
