use crate::components::TextReveal;
use crate::driver::VnStory;
use crate::render::{TransitionQueryItem, complete_transitions};
use crate::resources::{AssetLoadingState, PresentationCommandQueue};
use bevy::ecs::system::SystemParam;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use vn_core::VmEvent;
use vn_runtime::commands_from_event;

/// Menu choice requested by UI/input code.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct PendingChoice(pub usize);

/// Keyboard-focused menu choice.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct MenuFocus(pub usize);

#[derive(SystemParam)]
pub struct AdvanceInput<'w, 's> {
    keys: Option<Res<'w, ButtonInput<KeyCode>>>,
    mouse: Option<Res<'w, ButtonInput<MouseButton>>>,
    story: Option<ResMut<'w, VnStory>>,
    queue: ResMut<'w, PresentationCommandQueue>,
    loading: Res<'w, AssetLoadingState>,
    focus: ResMut<'w, MenuFocus>,
    transitions: Query<'w, 's, TransitionQueryItem<'static>>,
    reveals: Query<'w, 's, Entity, With<TextReveal>>,
}

/// Advances dialogue, completes effects, or navigates the active menu.
pub fn keyboard_advance_story(mut commands: Commands, input: AdvanceInput) {
    let AdvanceInput {
        keys,
        mouse,
        story,
        mut queue,
        loading,
        mut focus,
        transitions,
        reveals,
    } = input;
    if loading.started_at.is_some() || loading.error.is_some() {
        return;
    }
    let key_advance = keys
        .as_deref()
        .is_some_and(|keys| keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter));
    let click_advance = mouse
        .as_deref()
        .is_some_and(|mouse| mouse.just_pressed(MouseButton::Left));
    let Some(mut story) = story else {
        return;
    };
    if let Some(VmEvent::Menu { choices }) = story.last_event() {
        if !transitions.is_empty() {
            return;
        }
        let Some(keys) = keys.as_deref() else {
            return;
        };
        if keys.just_pressed(KeyCode::ArrowDown) {
            focus.0 = (focus.0 + 1) % choices.len();
        } else if keys.just_pressed(KeyCode::ArrowUp) {
            focus.0 = focus.0.checked_sub(1).unwrap_or(choices.len() - 1);
        } else if keys.just_pressed(KeyCode::Enter) {
            commands.insert_resource(PendingChoice(focus.0));
        } else if let Some(index) = number_choice(keys, choices.len()) {
            commands.insert_resource(PendingChoice(index));
        }
        return;
    }
    if !(key_advance || click_advance) {
        return;
    }
    let mut completed_effect = complete_transitions(&mut commands, transitions);
    for entity in &reveals {
        commands.entity(entity).remove::<TextReveal>();
        completed_effect = true;
    }
    if completed_effect {
        return;
    }
    if let Ok(event) = story.continue_story() {
        focus.0 = 0;
        queue_event_and_following_visuals(&mut story, &mut queue, event);
    }
}

fn number_choice(keys: &ButtonInput<KeyCode>, count: usize) -> Option<usize> {
    [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ]
    .into_iter()
    .take(count)
    .position(|key| keys.just_pressed(key))
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

pub(crate) fn queue_event_and_following_visuals(
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
