//! Bevy integration for VN presentation state.
//!
//! serializable presentation snapshot into marker/render entities that Bevy
//! renderer systems can consume.

pub mod camera;
pub mod components;
pub mod driver;
pub mod input;
#[cfg(feature = "desktop")]
pub mod player;
pub mod plugin;
pub mod render;
pub mod resources;
pub mod systems;

pub use camera::VnCamera;
pub use components::{
    PresentationBackground, PresentationDialogue, PresentationMenu, PresentationMusic,
    PresentationSprite, TextReveal, TransitionAlpha, TransitionFlags, TransitionPhase,
};
pub use driver::VnStory;
pub use input::{MenuFocus, PendingChoice, apply_pending_choice, keyboard_advance_story};
#[cfg(feature = "desktop")]
pub use player::{PlayerConfig, PlayerFlags, PlayerMode, run_player};
pub use plugin::VnBevyPlugin;
pub use render::{BackgroundRender, MusicRender, SpriteRender};
pub use resources::{
    AssetLoadingState, PresentationCommandQueue, VnAssetResolver, VnPresentation, VnRenderable,
};
#[cfg(feature = "audio")]
pub use systems::validate_audio;
