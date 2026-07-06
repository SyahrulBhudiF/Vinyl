use std::path::{Path, PathBuf};
use vn_script::{AssetId, ProjectManifest};

/// Resolves script-level asset identifiers to project-relative asset paths.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetResolver {
    inner: vn_script::AssetResolver,
}

impl AssetResolver {
    /// Creates a resolver rooted at a game project directory.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Self::with_manifest(root.clone(), ProjectManifest::default_for_root(&root))
    }

    /// Creates a resolver using manifest-configured asset roots.
    pub fn with_manifest(root: impl Into<PathBuf>, manifest: ProjectManifest) -> Self {
        Self {
            inner: vn_script::AssetResolver::new(root, manifest),
        }
    }

    /// Resolves `scene bg room` style identifiers to `assets/bg/room.png`.
    pub fn background(&self, image: &str) -> PathBuf {
        self.inner.resolve(&AssetId::Background(image.to_string()))
    }

    /// Resolves `show eileen happy` style identifiers to `assets/sprites/eileen/happy.png`.
    pub fn sprite(&self, tag: &str, attrs: &[String]) -> PathBuf {
        self.inner.resolve(&AssetId::Sprite {
            tag: tag.to_string(),
            attrs: attrs.to_vec(),
        })
    }

    /// Resolves audio paths relative to the project root.
    pub fn audio(&self, path: &str) -> PathBuf {
        self.inner.resolve(&AssetId::Audio(path.to_string()))
    }

    /// Returns the resolver root.
    pub fn root(&self) -> &Path {
        self.inner.root()
    }
}

/// Asset paths derived from a presentation snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedAssets {
    pub background: Option<PathBuf>,
    pub sprites: Vec<(String, PathBuf)>,
    pub music: Option<PathBuf>,
}

impl AssetResolver {
    /// Resolves every currently visible asset in presentation state.
    pub fn resolve_snapshot(&self, snapshot: &vn_core::PresentationSnapshot) -> ResolvedAssets {
        let background = snapshot
            .background
            .as_ref()
            .map(|image| self.background(image));
        let mut sprites = snapshot
            .sprites
            .iter()
            .map(|(tag, sprite)| (tag.clone(), self.sprite(tag, &sprite.attrs)))
            .collect::<Vec<_>>();
        sprites.sort_by(|left, right| left.0.cmp(&right.0));
        let music = snapshot.music.as_ref().map(|path| self.audio(path));
        ResolvedAssets {
            background,
            sprites,
            music,
        }
    }
}
