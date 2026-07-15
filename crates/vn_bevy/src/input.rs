use crate::components::{PresentationMusic, TextReveal};
use crate::driver::VnStory;
#[cfg(feature = "desktop")]
use crate::player::PlayerMode;
use crate::render::{TransitionQueryItem, complete_transitions};
use crate::resources::{AssetLoadingState, PresentationCommandQueue, VnPresentation};
use bevy::ecs::system::SystemParam;
use bevy::input::{ButtonInput, mouse::MouseWheel};
use bevy::prelude::*;
use vn_core::VmEvent;
use vn_runtime::commands_from_event;

/// Menu choice requested by UI/input code.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct PendingChoice(pub usize);

/// Rollback requested by player UI.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct PendingRollback;

/// Keyboard-focused menu choice.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct MenuFocus(pub usize);

/// Blocks the click that revealed the current menu from selecting it in the same press.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct MenuClickGuard(pub bool);

#[derive(SystemParam)]
pub struct AdvanceInput<'w, 's> {
    keys: Option<Res<'w, ButtonInput<KeyCode>>>,
    mouse: Option<Res<'w, ButtonInput<MouseButton>>>,
    story: Option<ResMut<'w, VnStory>>,
    queue: ResMut<'w, PresentationCommandQueue>,
    loading: Res<'w, AssetLoadingState>,
    #[cfg(feature = "desktop")]
    mode: Option<Res<'w, PlayerMode>>,
    focus: ResMut<'w, MenuFocus>,
    menu_click_guard: ResMut<'w, MenuClickGuard>,
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
        #[cfg(feature = "desktop")]
        mode,
        mut focus,
        mut menu_click_guard,
        transitions,
        reveals,
    } = input;
    if loading.started_at.is_some()
        || loading.error.is_some()
        || !player_accepts_story_input(
            #[cfg(feature = "desktop")]
            mode,
        )
    {
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
        if mouse
            .as_deref()
            .is_some_and(|mouse| mouse.just_released(MouseButton::Left))
        {
            menu_click_guard.0 = false;
        }
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
        let reached_menu = queue_event_and_following_visuals(&mut story, &mut queue, event);
        menu_click_guard.0 = click_advance && reached_menu;
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
        let _ = queue_event_and_following_visuals(&mut story, &mut queue, event);
    }
}

#[derive(SystemParam)]
pub struct RollbackInput<'w, 's> {
    keys: Option<Res<'w, ButtonInput<KeyCode>>>,
    mouse_wheel: Option<MessageReader<'w, 's, MouseWheel>>,
    pending: Option<Res<'w, PendingRollback>>,
    story: Option<ResMut<'w, VnStory>>,
    presentation: ResMut<'w, VnPresentation>,
    queue: ResMut<'w, PresentationCommandQueue>,
    loading: Res<'w, AssetLoadingState>,
    #[cfg(feature = "desktop")]
    mode: Option<Res<'w, PlayerMode>>,
    transitions: Query<'w, 's, (), With<crate::components::TransitionAlpha>>,
    music: Query<'w, 's, Entity, With<PresentationMusic>>,
}

/// Rolls back one stable interaction checkpoint with PageUp or mouse wheel up.
pub fn rollback_story(mut commands: Commands, input: RollbackInput) {
    let RollbackInput {
        keys,
        mouse_wheel,
        pending,
        story,
        mut presentation,
        mut queue,
        loading,
        #[cfg(feature = "desktop")]
        mode,
        transitions,
        music,
    } = input;
    if loading.started_at.is_some()
        || loading.error.is_some()
        || (!player_accepts_story_input(
            #[cfg(feature = "desktop")]
            mode,
        ) && pending.is_none())
        || !transitions.is_empty()
    {
        return;
    }
    let page_up = keys
        .as_deref()
        .is_some_and(|keys| keys.just_pressed(KeyCode::PageUp));
    let wheel_up = mouse_wheel.is_some_and(|mut events| events.read().any(|event| event.y > 0.0));
    let ui_requested = pending.is_some();
    if ui_requested {
        commands.remove_resource::<PendingRollback>();
    }
    if !(page_up || wheel_up || ui_requested) {
        return;
    }
    let Some(mut story) = story else {
        return;
    };
    if story.rollback().is_some() {
        queue.commands.clear();
        presentation.snapshot = story.vm().presentation().clone();
        presentation.pending_commands.clear();
        for entity in &music {
            commands.entity(entity).despawn();
        }
    }
}

fn player_accepts_story_input(#[cfg(feature = "desktop")] mode: Option<Res<PlayerMode>>) -> bool {
    #[cfg(feature = "desktop")]
    {
        mode.is_none_or(|mode| *mode == PlayerMode::Playing)
    }
    #[cfg(not(feature = "desktop"))]
    {
        true
    }
}

pub(crate) fn queue_event_and_following_visuals(
    story: &mut VnStory,
    queue: &mut PresentationCommandQueue,
    event: VmEvent,
) -> bool {
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
            return matches!(event, VmEvent::Menu { .. });
        }
        let Ok(next) = story.continue_story() else {
            return false;
        };
        event = next;
    }
}
