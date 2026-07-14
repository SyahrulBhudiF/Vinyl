use bevy::asset::UntypedHandle;
use bevy::prelude::*;
use std::collections::VecDeque;
use std::time::Instant;
use vn_runtime::{AssetResolver, PresentationCommand};
use vn_script::ProjectManifest;

/// Bevy resource containing serializable VN presentation state.
#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct VnPresentation {
    pub snapshot: vn_core::PresentationSnapshot,
    pub pending_commands: Vec<PresentationCommand>,
}

impl std::ops::Deref for VnPresentation {
    type Target = vn_core::PresentationSnapshot;

    fn deref(&self) -> &Self::Target {
        &self.snapshot
    }
}

impl std::ops::DerefMut for VnPresentation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.snapshot
    }
}

/// FIFO command queue from runtime orchestration into Bevy systems.
#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct PresentationCommandQueue {
    pub(crate) commands: VecDeque<PresentationCommand>,
}

impl PresentationCommandQueue {
    /// Enqueues one renderer-independent presentation command.
    pub fn push(&mut self, command: PresentationCommand) {
        self.commands.push_back(command);
    }

    /// Returns true when no commands are pending.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

/// Current asynchronous asset-loading state.
#[derive(Clone, Debug, Default, Resource)]
pub struct AssetLoadingState {
    pub started_at: Option<Instant>,
    pub error: Option<String>,
    pub(crate) pending_path: Option<String>,
    pub(crate) pending_handle: Option<UntypedHandle>,
}

impl AssetLoadingState {
    pub fn visible(&self) -> bool {
        self.started_at
            .is_some_and(|started| started.elapsed().as_millis() >= 150)
    }
}

/// Project-rooted resolver used by Bevy asset-loading systems.
#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct VnAssetResolver(pub AssetResolver);

impl VnAssetResolver {
    /// Creates a resolver rooted at a game project directory.
    pub fn new(root: impl Into<std::path::PathBuf>) -> Self {
        Self(AssetResolver::new(root))
    }

    /// Creates a resolver using manifest-configured asset roots.
    pub fn with_manifest(root: impl Into<std::path::PathBuf>, manifest: ProjectManifest) -> Self {
        Self(AssetResolver::with_manifest(root, manifest))
    }

    pub(crate) fn asset_path(&self, path: std::path::PathBuf) -> String {
        path.strip_prefix(self.0.root())
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::VnAssetResolver;

    #[test]
    fn produces_project_relative_bevy_asset_paths() {
        let resolver = VnAssetResolver::new("game");

        assert_eq!(
            resolver.asset_path(resolver.0.background("bg school hallway")),
            "assets/bg/school/hallway.png"
        );
        assert_eq!(
            resolver.asset_path(resolver.0.sprite("eileen", &["happy".to_string()])),
            "assets/sprites/eileen/happy.png"
        );
    }
}

/// Enables sprite/background render entity materialization.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct VnRenderable(pub bool);
