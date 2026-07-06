//! Bevy integration for VN presentation state.
//!
//! serializable presentation snapshot into marker/render entities that Bevy
//! renderer systems can consume.

pub mod camera;
pub mod components;
pub mod driver;
pub mod input;
pub mod plugin;
pub mod render;
pub mod resources;
pub mod systems;

pub use camera::VnCamera;
pub use components::{
    PresentationBackground, PresentationDialogue, PresentationMenu, PresentationMusic,
    PresentationSprite, TextReveal, TransitionAlpha,
};
pub use driver::VnStory;
pub use input::{PendingChoice, apply_pending_choice, keyboard_advance_story};
pub use plugin::VnBevyPlugin;
pub use render::{BackgroundRender, MusicRender, SpriteRender};
pub use resources::{PresentationCommandQueue, VnAssetResolver, VnPresentation, VnRenderable};
