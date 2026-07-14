use crate::camera::spawn_camera;
use crate::input::{MenuFocus, apply_pending_choice, keyboard_advance_story};
use crate::render::{fit_loaded_sprites, sync_render_entities};
use crate::resources::{AssetLoadingState, PresentationCommandQueue, VnPresentation, VnRenderable};
use crate::systems::{
    apply_queued_commands, sync_presentation_entities, tick_text_reveal, tick_transition_alpha,
};
use bevy::prelude::*;

/// Bevy plugin that owns renderer-facing VN presentation resources.
pub struct VnBevyPlugin;

impl Plugin for VnBevyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VnPresentation>()
            .init_resource::<PresentationCommandQueue>()
            .init_resource::<AssetLoadingState>()
            .init_resource::<MenuFocus>()
            .init_resource::<VnRenderable>()
            .init_resource::<Time>()
            .add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                (
                    keyboard_advance_story,
                    apply_pending_choice,
                    apply_queued_commands,
                    ApplyDeferred,
                    sync_presentation_entities,
                    ApplyDeferred,
                    sync_render_entities,
                    fit_loaded_sprites,
                    tick_transition_alpha,
                    tick_text_reveal,
                )
                    .chain(),
            );
    }
}
