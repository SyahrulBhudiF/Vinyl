//! Runtime orchestration between story VM events and presentation state.

pub mod asset;

pub mod presentation;
pub mod save_store;

pub use asset::{AssetResolver, ResolvedAssets};
pub use presentation::{PresentationCommand, apply_command, commands_from_event};
pub use save_store::{
    SaveSlot, SaveSlotState, inspect_save, project_save_dir, read_save, write_save,
};
