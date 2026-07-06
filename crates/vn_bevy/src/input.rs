use crate::driver::VnStory;
use crate::resources::PresentationCommandQueue;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use vn_core::VmEvent;
use vn_runtime::commands_from_event;

/// Menu choice requested by UI/input code.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct PendingChoice(pub usize);

/// Advances the story on Space/Enter when no menu is active.
pub fn keyboard_advance_story(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    story: Option<ResMut<VnStory>>,
    mut queue: ResMut<PresentationCommandQueue>,
) {
    let Some(keys) = keys else {
        return;
    };
    if !(keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter)) {
        return;
    }
    let Some(mut story) = story else {
        return;
    };
    if matches!(story.last_event(), Some(VmEvent::Menu { .. })) {
        return;
    }
    if let Ok(event) = story.continue_story() {
        queue_event_and_following_visuals(&mut story, &mut queue, event);
    }
}

/// Chooses a pending menu option and queues resulting presentation commands.
pub fn apply_pending_choice(
    mut commands: Commands,
    pending: Option<Res<PendingChoice>>,
    story: Option<ResMut<VnStory>>,
    mut queue: ResMut<PresentationCommandQueue>,
) {
    let Some(pending) = pending else {
        return;
    };
    let choice = pending.0;
    commands.remove_resource::<PendingChoice>();
    let Some(mut story) = story else {
        return;
    };
    if let Ok(event) = story.choose(choice) {
        queue_event_and_following_visuals(&mut story, &mut queue, event);
    }
}

fn queue_event_and_following_visuals(
    story: &mut VnStory,
    queue: &mut PresentationCommandQueue,
    event: VmEvent,
) {
    let mut event = event;
    loop {
        let should_continue = matches!(
            event,
            VmEvent::Scene { .. }
                | VmEvent::Show { .. }
                | VmEvent::Hide { .. }
                | VmEvent::PlayMusic { .. }
                | VmEvent::StopMusic
        );
        for command in commands_from_event(&event) {
            queue.push(command);
        }
        if !should_continue {
            break;
        }
        let Ok(next) = story.continue_story() else {
            break;
        };
        event = next;
    }
}
