use bevy::prelude::*;
use std::collections::VecDeque;
use vn_runtime::{AssetResolver, PresentationCommand};
use vn_script::ProjectManifest;

/// Bevy resource containing serializable VN presentation state.
#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct VnPresentation(pub vn_core::PresentationSnapshot);

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
}

/// Enables sprite/background render entity materialization.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct VnRenderable(pub bool);
