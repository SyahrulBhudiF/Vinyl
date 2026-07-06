use crate::components::{
    PresentationBackground, PresentationDialogue, PresentationMenu, PresentationMusic,
    PresentationSprite, TextReveal, TransitionAlpha,
};
use crate::resources::{PresentationCommandQueue, VnPresentation};
use bevy::prelude::*;
use std::collections::HashSet;
use vn_core::TextEffect;
use vn_runtime::{PresentationCommand, apply_command};

pub(crate) fn apply_queued_commands(
    mut queue: ResMut<PresentationCommandQueue>,
    mut presentation: ResMut<VnPresentation>,
) {
    while let Some(command) = queue.commands.pop_front() {
        presentation.pending_commands.push(command.clone());
        apply_command(&mut presentation.snapshot, &command);
    }
}

pub(crate) fn sync_presentation_entities(
    mut commands: Commands,
    mut presentation: ResMut<VnPresentation>,
    backgrounds: Query<(Entity, &PresentationBackground)>,
    sprites: Query<(Entity, &PresentationSprite)>,
    dialogues: Query<(Entity, &PresentationDialogue)>,
    menus: Query<(Entity, &PresentationMenu)>,
    music: Query<(Entity, &PresentationMusic)>,
) {
    let pending_commands = std::mem::take(&mut presentation.pending_commands);
    sync_background(
        &mut commands,
        &presentation,
        &pending_commands,
        &backgrounds,
    );
    sync_sprites(&mut commands, &presentation, &pending_commands, &sprites);
    sync_dialogue(&mut commands, &presentation, &pending_commands, &dialogues);
    sync_menu(&mut commands, &presentation, &menus);
    sync_music_marker(&mut commands, &presentation, &music);
}

fn sync_background(
    commands: &mut Commands,
    presentation: &VnPresentation,
    pending_commands: &[PresentationCommand],
    backgrounds: &Query<(Entity, &PresentationBackground)>,
) {
    match &presentation.snapshot.background {
        Some(image) => {
            let keep = backgrounds
                .iter()
                .find(|(_, background)| background.image == *image)
                .map(|(entity, _)| entity);
            for (entity, _) in backgrounds.iter() {
                if Some(entity) != keep {
                    commands.entity(entity).despawn();
                }
            }
            if keep.is_none() {
                let transition = pending_commands
                    .iter()
                    .rev()
                    .find_map(|command| match command {
                        PresentationCommand::SetBackground { transition, .. } => transition.clone(),
                        _ => None,
                    });
                let alpha = transition_alpha(transition.clone());
                let mut entity = commands.spawn(PresentationBackground {
                    image: image.clone(),
                    transition,
                });
                if let Some(alpha) = alpha {
                    entity.insert(alpha);
                }
            }
        }
        None => despawn_all(commands, backgrounds.iter().map(|(entity, _)| entity)),
    }
}

fn sync_sprites(
    commands: &mut Commands,
    presentation: &VnPresentation,
    pending_commands: &[PresentationCommand],
    sprites: &Query<(Entity, &PresentationSprite)>,
) {
    let live = presentation
        .snapshot
        .sprites
        .keys()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    for (entity, sprite) in sprites.iter() {
        let changed = presentation
            .snapshot
            .sprites
            .get(&sprite.tag)
            .is_some_and(|next| next.attrs != sprite.attrs || next.position != sprite.position);
        if !live.contains(sprite.tag.as_str()) || changed {
            commands.entity(entity).despawn();
        }
    }

    let mut desired = presentation.snapshot.sprites.iter().collect::<Vec<_>>();
    desired.sort_by(|left, right| left.0.cmp(right.0));
    for (tag, sprite) in desired {
        let exists = sprites.iter().any(|(_, old)| {
            old.tag == *tag && old.attrs == sprite.attrs && old.position == sprite.position
        });
        if exists {
            continue;
        }
        let transition = pending_commands
            .iter()
            .rev()
            .find_map(|command| match command {
                PresentationCommand::ShowSprite {
                    tag: changed_tag,
                    transition,
                    ..
                } if changed_tag == tag => transition.clone(),
                _ => None,
            });
        let alpha = transition_alpha(transition.clone());
        let mut entity = commands.spawn(PresentationSprite {
            tag: tag.clone(),
            attrs: sprite.attrs.clone(),
            position: sprite.position.clone(),
            transition,
        });
        if let Some(alpha) = alpha {
            entity.insert(alpha);
        }
    }
}

fn sync_dialogue(
    commands: &mut Commands,
    presentation: &VnPresentation,
    pending_commands: &[PresentationCommand],
    dialogues: &Query<(Entity, &PresentationDialogue)>,
) {
    match &presentation.snapshot.dialogue {
        Some(dialogue) => {
            let keep = dialogues
                .iter()
                .find(|(_, old)| old.speaker == dialogue.speaker && old.text == dialogue.text)
                .map(|(entity, _)| entity);
            for (entity, _) in dialogues.iter() {
                if Some(entity) != keep {
                    commands.entity(entity).despawn();
                }
            }
            if keep.is_none() {
                let effect = pending_commands
                    .iter()
                    .rev()
                    .find_map(|command| match command {
                        PresentationCommand::ShowDialogue { effect, .. } => Some(effect.clone()),
                        _ => None,
                    })
                    .unwrap_or(TextEffect::Instant);
                let reveal = text_reveal(&dialogue.text, effect.clone());
                let mut entity = commands.spawn(PresentationDialogue {
                    speaker: dialogue.speaker.clone(),
                    text: dialogue.text.clone(),
                    effect,
                });
                if let Some(reveal) = reveal {
                    entity.insert(reveal);
                }
            }
        }
        None => despawn_all(commands, dialogues.iter().map(|(entity, _)| entity)),
    }
}

fn sync_menu(
    commands: &mut Commands,
    presentation: &VnPresentation,
    menus: &Query<(Entity, &PresentationMenu)>,
) {
    match &presentation.snapshot.menu {
        Some(menu) => {
            let keep = menus
                .iter()
                .find(|(_, old)| old.choices == *menu)
                .map(|(entity, _)| entity);
            for (entity, _) in menus.iter() {
                if Some(entity) != keep {
                    commands.entity(entity).despawn();
                }
            }
            if keep.is_none() {
                commands.spawn(PresentationMenu {
                    choices: menu.clone(),
                });
            }
        }
        None => despawn_all(commands, menus.iter().map(|(entity, _)| entity)),
    }
}

fn sync_music_marker(
    commands: &mut Commands,
    presentation: &VnPresentation,
    music: &Query<(Entity, &PresentationMusic)>,
) {
    match &presentation.snapshot.music {
        Some(path) => {
            let keep = music
                .iter()
                .find(|(_, old)| old.path == *path)
                .map(|(entity, _)| entity);
            for (entity, _) in music.iter() {
                if Some(entity) != keep {
                    commands.entity(entity).despawn();
                }
            }
            if keep.is_none() {
                commands.spawn(PresentationMusic { path: path.clone() });
            }
        }
        None => despawn_all(commands, music.iter().map(|(entity, _)| entity)),
    }
}

pub(crate) fn tick_transition_alpha(
    time: Res<Time>,
    mut commands: Commands,
    mut transitions: Query<(Entity, &mut TransitionAlpha, Option<&mut Sprite>)>,
) {
    let delta_ms = time.delta().as_millis().min(u128::from(u32::MAX)) as u32;
    for (entity, mut transition, sprite) in &mut transitions {
        transition.elapsed_ms = transition.elapsed_ms.saturating_add(delta_ms);
        let alpha = transition.alpha_permille() as f32 / 1000.0;
        if let Some(mut sprite) = sprite {
            sprite.color = sprite.color.with_alpha(alpha);
        }
        if transition.elapsed_ms >= transition.duration_ms {
            commands.entity(entity).remove::<TransitionAlpha>();
        }
    }
}

pub(crate) fn tick_text_reveal(
    time: Res<Time>,
    mut commands: Commands,
    mut reveals: Query<(Entity, &mut TextReveal)>,
) {
    let delta_ms = time.delta().as_millis().min(u128::from(u32::MAX)) as u32;
    for (entity, mut reveal) in &mut reveals {
        reveal.elapsed_ms = reveal.elapsed_ms.saturating_add(delta_ms);
        if reveal.visible_chars() >= reveal.total_chars {
            commands.entity(entity).remove::<TextReveal>();
        }
    }
}

fn transition_alpha(transition: Option<vn_core::Transition>) -> Option<TransitionAlpha> {
    let duration_ms = transition?.duration_ms;
    (duration_ms > 0).then_some(TransitionAlpha {
        elapsed_ms: 0,
        duration_ms,
    })
}

fn text_reveal(text: &str, effect: TextEffect) -> Option<TextReveal> {
    match effect {
        TextEffect::Instant => None,
        TextEffect::Typewriter { chars_per_second } => Some(TextReveal {
            elapsed_ms: 0,
            chars_per_second,
            total_chars: text.chars().count(),
        }),
    }
}

fn despawn_all(commands: &mut Commands, entities: impl Iterator<Item = Entity>) {
    for entity in entities {
        commands.entity(entity).despawn();
    }
}
