use crate::components::{
    PresentationBackground, PresentationDialogue, PresentationMenu, PresentationMusic,
    PresentationSprite, TextReveal, TransitionAlpha,
};
use crate::resources::{
    AssetLoadingState, PresentationCommandQueue, VnAssetResolver, VnPresentation,
};
use bevy::asset::LoadState;
use bevy::prelude::*;
use std::collections::HashSet;
#[cfg(feature = "audio")]
use std::fs::File;
#[cfg(feature = "audio")]
use std::io::BufReader;
use std::time::Instant;
use vn_core::TextEffect;
use vn_runtime::{PresentationCommand, apply_command};

pub(crate) fn apply_queued_commands(
    mut queue: ResMut<PresentationCommandQueue>,
    mut presentation: ResMut<VnPresentation>,
    asset_server: Option<Res<AssetServer>>,
    resolver: Option<Res<VnAssetResolver>>,
    mut loading: ResMut<AssetLoadingState>,
) {
    while let Some(command) = queue.commands.front() {
        match command_asset_state(
            command,
            asset_server.as_deref(),
            resolver.as_deref(),
            &mut loading,
        ) {
            CommandAssetState::Ready => {
                let command = queue.commands.pop_front().expect("front command exists");
                #[cfg(feature = "audio")]
                if let PresentationCommand::PlayMusic(path) = &command
                    && let Some(resolver) = resolver.as_deref()
                    && resolver.0.audio(path).exists()
                    && let Err(error) = validate_audio(&resolver.0.audio(path))
                {
                    eprintln!("asset load failed: {error}");
                    loading.error = Some(error);
                    loading.started_at = None;
                    loading.pending_path = None;
                    loading.pending_handle = None;
                    queue.commands.clear();
                    break;
                }
                presentation.pending_commands.push(command.clone());
                apply_command(&mut presentation.snapshot, &command);
                loading.started_at = None;
                loading.pending_path = None;
                loading.pending_handle = None;
            }
            CommandAssetState::Loading => {
                loading.started_at.get_or_insert_with(Instant::now);
                break;
            }
            CommandAssetState::Failed(error) => {
                eprintln!("asset load failed: {error}");
                loading.error = Some(error);
                loading.started_at = None;
                loading.pending_path = None;
                loading.pending_handle = None;
                queue.commands.clear();
                break;
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum CommandAssetState {
    Ready,
    Loading,
    Failed(String),
}

fn command_asset_state(
    command: &PresentationCommand,
    asset_server: Option<&AssetServer>,
    resolver: Option<&VnAssetResolver>,
    loading: &mut AssetLoadingState,
) -> CommandAssetState {
    let Some(path) = command_asset_path(command, resolver) else {
        return CommandAssetState::Ready;
    };
    let Some(asset_server) = asset_server else {
        return CommandAssetState::Ready;
    };
    if loading.pending_path.as_deref() != Some(&path) {
        loading.pending_handle = Some(asset_server.load_untyped(path.clone()).untyped());
        loading.pending_path = Some(path.clone());
    }
    let Some(handle) = loading.pending_handle.as_ref() else {
        return CommandAssetState::Loading;
    };
    match asset_server.get_load_state(handle.id()) {
        Some(LoadState::Loaded) => CommandAssetState::Ready,
        Some(LoadState::Failed(error)) => CommandAssetState::Failed(format!("{path}: {error}")),
        _ => CommandAssetState::Loading,
    }
}

#[cfg(feature = "audio")]
pub fn validate_audio(path: &std::path::Path) -> Result<(), String> {
    let file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    rodio::Decoder::new(BufReader::new(file))
        .map(|_| ())
        .map_err(|error| format!("{}: {error}", path.display()))
}

fn command_asset_path(
    command: &PresentationCommand,
    resolver: Option<&VnAssetResolver>,
) -> Option<String> {
    let resolver = resolver?;
    let path = match command {
        PresentationCommand::SetBackground { image, .. } => resolver.0.background(image),
        PresentationCommand::ShowSprite { tag, attrs, .. } => resolver.0.sprite(tag, attrs),
        #[cfg(feature = "audio")]
        PresentationCommand::PlayMusic(path) => resolver.0.audio(path),
        #[cfg(not(feature = "audio"))]
        PresentationCommand::PlayMusic(_) => return None,
        _ => return None,
    };
    Some(resolver.asset_path(path))
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
