use crate::components::{
    PresentationBackground, PresentationMusic, PresentationSprite, TransitionAlpha,
    TransitionFlags, TransitionPhase,
};
use crate::resources::{VnAssetResolver, VnRenderable};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::sprite::{ScalingMode, SpriteImageMode};
use std::collections::HashSet;

/// Render marker for a background sprite entity.
#[derive(Clone, Debug, Eq, PartialEq, Component)]
pub struct BackgroundRender {
    pub image: String,
}

/// Render marker for sprite entities.
#[derive(Clone, Debug, Eq, PartialEq, Component)]
pub struct SpriteRender {
    pub tag: String,
    pub attrs: Vec<String>,
    pub position: String,
}

pub(crate) type TransitionQueryItem<'a> = (
    Entity,
    &'a TransitionAlpha,
    &'a TransitionFlags,
    Option<&'a mut Sprite>,
    Option<&'a mut Visibility>,
);

type TransitionQueryItemMut<'a> = (
    Entity,
    &'a mut TransitionAlpha,
    &'a mut TransitionFlags,
    Option<&'a mut Sprite>,
    Option<&'a mut Visibility>,
);

/// Audio marker for the active music entity.
#[derive(Clone, Debug, Eq, PartialEq, Component)]
pub struct MusicRender {
    pub path: String,
}

#[derive(SystemParam)]
pub(crate) struct RenderPresentationQueries<'w, 's> {
    backgrounds: Query<'w, 's, &'static PresentationBackground>,
    sprites: Query<'w, 's, &'static PresentationSprite>,
    music: Query<'w, 's, &'static PresentationMusic>,
    background_renders: Query<'w, 's, (Entity, &'static BackgroundRender)>,
    sprite_renders: Query<'w, 's, (Entity, &'static SpriteRender)>,
    music_renders: Query<'w, 's, (Entity, &'static MusicRender)>,
}

pub(crate) fn sync_render_entities(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    resolver: Option<Res<VnAssetResolver>>,
    renderable: Res<VnRenderable>,
    presentation: RenderPresentationQueries,
) {
    if !renderable.0 {
        return;
    }

    sync_background(&mut commands, &asset_server, &resolver, &presentation);
    sync_music(&mut commands, &asset_server, &resolver, &presentation);
    sync_sprites(&mut commands, &asset_server, &resolver, &presentation);
}

fn sync_background(
    commands: &mut Commands,
    asset_server: &Option<Res<AssetServer>>,
    resolver: &Option<Res<VnAssetResolver>>,
    presentation: &RenderPresentationQueries,
) {
    let background = presentation.backgrounds.iter().next();
    let Some(background) = background else {
        for (entity, _) in presentation.background_renders.iter() {
            commands.entity(entity).despawn();
        }
        return;
    };
    let image = background.image.clone();
    if presentation
        .background_renders
        .iter()
        .any(|(_, render)| render.image == image)
    {
        return;
    }
    let old = presentation
        .background_renders
        .iter()
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    let transition = transition_kind(background.transition.as_ref());
    match transition {
        Some(("fade", duration_ms)) => {
            for entity in old {
                commands.entity(entity).insert((
                    transition_alpha(duration_ms, TransitionPhase::FadeOut),
                    TransitionFlags {
                        despawn_after: true,
                        await_fade_in: false,
                    },
                ));
            }
            let mut incoming = spawn_background(commands, asset_server, resolver, &image);
            incoming.insert((
                transition_alpha(duration_ms, TransitionPhase::FadeIn),
                TransitionFlags {
                    despawn_after: false,
                    await_fade_in: true,
                },
                Visibility::Hidden,
            ));
        }
        Some(("dissolve", duration_ms)) => {
            for entity in old {
                commands.entity(entity).insert((
                    transition_alpha(duration_ms, TransitionPhase::DissolveOut),
                    TransitionFlags {
                        despawn_after: true,
                        await_fade_in: false,
                    },
                ));
            }
            spawn_background(commands, asset_server, resolver, &image).insert((
                transition_alpha(duration_ms, TransitionPhase::DissolveIn),
                TransitionFlags::default(),
            ));
        }
        _ => {
            for entity in old {
                commands.entity(entity).despawn();
            }
            spawn_background(commands, asset_server, resolver, &image);
        }
    }
}

fn spawn_background<'a>(
    commands: &'a mut Commands,
    asset_server: &Option<Res<AssetServer>>,
    resolver: &Option<Res<VnAssetResolver>>,
    image: &str,
) -> EntityCommands<'a> {
    commands.spawn((
        BackgroundRender {
            image: image.to_string(),
        },
        Sprite {
            image: load_image(asset_server, resolver, |resolver| {
                resolver.0.background(image)
            }),
            custom_size: Some(Vec2::new(1280.0, 720.0)),
            image_mode: SpriteImageMode::Scale(ScalingMode::FillCenter),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -10.0),
    ))
}

fn sync_music(
    commands: &mut Commands,
    asset_server: &Option<Res<AssetServer>>,
    resolver: &Option<Res<VnAssetResolver>>,
    presentation: &RenderPresentationQueries,
) {
    let path = presentation
        .music
        .iter()
        .next()
        .map(|music| music.path.clone());
    match path {
        Some(path) => {
            let existing = presentation.music_renders.iter().collect::<Vec<_>>();
            let keep = existing
                .iter()
                .find(|(_, render)| render.path == path)
                .map(|(entity, _)| *entity);
            for (entity, _) in existing {
                if Some(entity) != keep {
                    commands.entity(entity).despawn();
                }
            }
            if keep.is_none() {
                commands.spawn((
                    MusicRender { path: path.clone() },
                    audio_bundle(asset_server, resolver, |resolver| resolver.0.audio(&path)),
                ));
            }
        }
        None => {
            for (entity, _) in presentation.music_renders.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn sync_sprites(
    commands: &mut Commands,
    asset_server: &Option<Res<AssetServer>>,
    resolver: &Option<Res<VnAssetResolver>>,
    presentation: &RenderPresentationQueries,
) {
    let sprites = presentation.sprites.iter().collect::<Vec<_>>();
    let live_tags = sprites
        .iter()
        .map(|sprite| sprite.tag.as_str())
        .collect::<HashSet<_>>();

    for (entity, render) in presentation.sprite_renders.iter() {
        if !live_tags.contains(render.tag.as_str()) {
            commands.entity(entity).despawn();
        }
    }

    for sprite in sprites {
        if presentation.sprite_renders.iter().any(|(_, render)| {
            render.tag == sprite.tag
                && render.attrs == sprite.attrs
                && render.position == sprite.position
        }) {
            continue;
        }
        let old = presentation
            .sprite_renders
            .iter()
            .filter(|(_, render)| render.tag == sprite.tag)
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>();
        let transition = transition_kind(sprite.transition.as_ref());
        match transition {
            Some(("fade", duration_ms)) => {
                for entity in old {
                    commands.entity(entity).insert((
                        transition_alpha(duration_ms, TransitionPhase::FadeOut),
                        TransitionFlags {
                            despawn_after: true,
                            await_fade_in: false,
                        },
                    ));
                }
                spawn_sprite(commands, asset_server, resolver, sprite).insert((
                    transition_alpha(duration_ms, TransitionPhase::FadeIn),
                    TransitionFlags {
                        despawn_after: false,
                        await_fade_in: true,
                    },
                    Visibility::Hidden,
                ));
            }
            Some(("dissolve", duration_ms)) => {
                for entity in old {
                    commands.entity(entity).insert((
                        transition_alpha(duration_ms, TransitionPhase::DissolveOut),
                        TransitionFlags {
                            despawn_after: true,
                            await_fade_in: false,
                        },
                    ));
                }
                spawn_sprite(commands, asset_server, resolver, sprite).insert((
                    transition_alpha(duration_ms, TransitionPhase::DissolveIn),
                    TransitionFlags::default(),
                ));
            }
            _ => {
                for entity in old {
                    commands.entity(entity).despawn();
                }
                spawn_sprite(commands, asset_server, resolver, sprite);
            }
        }
    }
}

fn spawn_sprite<'a>(
    commands: &'a mut Commands,
    asset_server: &Option<Res<AssetServer>>,
    resolver: &Option<Res<VnAssetResolver>>,
    sprite: &PresentationSprite,
) -> EntityCommands<'a> {
    commands.spawn((
        SpriteRender {
            tag: sprite.tag.clone(),
            attrs: sprite.attrs.clone(),
            position: sprite.position.clone(),
        },
        Sprite::from_image(load_image(asset_server, resolver, |resolver| {
            resolver.0.sprite(&sprite.tag, &sprite.attrs)
        })),
        Transform::from_translation(position_to_translation(&sprite.position)),
    ))
}

pub(crate) fn tick_transition_alpha(
    time: Res<Time>,
    mut commands: Commands,
    mut transitions: Query<TransitionQueryItemMut>,
) {
    let delta_ms = time.delta().as_millis().min(u128::from(u32::MAX)) as u32;
    let fade_out_active = transitions.iter().any(|(_, transition, _, _, _)| {
        transition.phase == TransitionPhase::FadeOut
            && transition.elapsed_ms.saturating_add(delta_ms) < transition.duration_ms
    });
    for (entity, mut transition, mut flags, sprite, visibility) in &mut transitions {
        if flags.await_fade_in && fade_out_active {
            continue;
        }
        if flags.await_fade_in {
            flags.await_fade_in = false;
            if let Some(mut visibility) = visibility {
                *visibility = Visibility::Visible;
            }
        }
        transition.elapsed_ms = transition.elapsed_ms.saturating_add(delta_ms);
        if let Some(mut sprite) = sprite {
            sprite.color = sprite
                .color
                .with_alpha(transition.alpha_permille() as f32 / 1000.0);
        }
        if transition.elapsed_ms >= transition.duration_ms {
            if flags.despawn_after {
                commands.entity(entity).despawn();
            } else {
                commands.entity(entity).remove::<TransitionAlpha>();
                commands.entity(entity).remove::<TransitionFlags>();
            }
        }
    }
}

pub(crate) fn complete_transitions(
    commands: &mut Commands,
    mut transitions: Query<TransitionQueryItem>,
) -> bool {
    if transitions.is_empty() {
        return false;
    }
    for (entity, transition, flags, sprite, visibility) in &mut transitions {
        if flags.despawn_after {
            commands.entity(entity).despawn();
            continue;
        }
        if let Some(mut sprite) = sprite {
            sprite.color = sprite.color.with_alpha(1.0);
        }
        if flags.await_fade_in
            && let Some(mut visibility) = visibility
        {
            *visibility = Visibility::Visible;
        }
        let _ = transition;
        commands.entity(entity).remove::<TransitionAlpha>();
        commands.entity(entity).remove::<TransitionFlags>();
    }
    true
}

pub(crate) fn fit_loaded_sprites(
    images: Option<Res<Assets<Image>>>,
    mut sprites: Query<(&mut Sprite, &mut Transform), With<SpriteRender>>,
) {
    let Some(images) = images else {
        return;
    };
    for (mut sprite, mut transform) in &mut sprites {
        let Some(image) = images.get(&sprite.image) else {
            continue;
        };
        let size = image.size_f32();
        let scale = (648.0 / size.y).min(1.0);
        let displayed = size * scale;
        sprite.custom_size = (scale < 1.0).then_some(displayed);
        transform.translation.y = -360.0 + displayed.y / 2.0;
    }
}

fn load_image(
    asset_server: &Option<Res<AssetServer>>,
    resolver: &Option<Res<VnAssetResolver>>,
    path: impl FnOnce(&VnAssetResolver) -> std::path::PathBuf,
) -> Handle<Image> {
    match (asset_server, resolver) {
        (Some(asset_server), Some(resolver)) => {
            asset_server.load(resolver.asset_path(path(resolver)))
        }
        _ => Handle::<Image>::default(),
    }
}

#[cfg(feature = "audio")]
fn audio_bundle(
    asset_server: &Option<Res<AssetServer>>,
    resolver: &Option<Res<VnAssetResolver>>,
    path: impl FnOnce(&VnAssetResolver) -> std::path::PathBuf,
) -> (AudioPlayer<AudioSource>, PlaybackSettings) {
    let handle = match (asset_server, resolver) {
        (Some(asset_server), Some(resolver)) => {
            asset_server.load(resolver.asset_path(path(resolver)))
        }
        _ => Handle::<AudioSource>::default(),
    };
    (AudioPlayer::new(handle), PlaybackSettings::LOOP)
}

#[cfg(not(feature = "audio"))]
fn audio_bundle(
    _asset_server: &Option<Res<AssetServer>>,
    _resolver: &Option<Res<VnAssetResolver>>,
    _path: impl FnOnce(&VnAssetResolver) -> std::path::PathBuf,
) {
}

fn transition_kind(transition: Option<&vn_core::Transition>) -> Option<(&str, u32)> {
    let transition = transition?;
    (transition.duration_ms > 0).then_some((transition.kind.as_str(), transition.duration_ms))
}

fn transition_alpha(duration_ms: u32, phase: TransitionPhase) -> TransitionAlpha {
    TransitionAlpha {
        elapsed_ms: 0,
        duration_ms,
        phase,
    }
}

fn position_to_translation(position: &str) -> Vec3 {
    match position {
        "left" => Vec3::new(-320.0, -80.0, 0.0),
        "right" => Vec3::new(320.0, -80.0, 0.0),
        _ => Vec3::new(0.0, -80.0, 0.0),
    }
}
