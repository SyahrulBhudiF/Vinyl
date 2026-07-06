//! Runtime orchestration between story VM events and presentation state.

pub mod asset;

pub mod presentation;

pub use asset::{AssetResolver, ResolvedAssets};
pub use presentation::{PresentationCommand, apply_command, commands_from_event};
