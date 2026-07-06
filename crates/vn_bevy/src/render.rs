use crate::components::{
    PresentationBackground, PresentationMusic, PresentationSprite, TransitionAlpha,
};
use crate::resources::{VnAssetResolver, VnRenderable};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::HashSet;

/// Render marker for the active background sprite entity.
#[derive(Clone, Debug, Eq, PartialEq, Component)]
pub struct BackgroundRender {
    pub image: String,
}

/// Render marker for visible sprite entities.
#[derive(Clone, Debug, Eq, PartialEq, Component)]
pub struct SpriteRender {
    pub tag: String,
    pub attrs: Vec<String>,
    pub position: String,
}

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

    match background {
        Some(background) => {
            let image = background.image.clone();
            let existing = presentation.background_renders.iter().collect::<Vec<_>>();
            let keep = existing
                .iter()
                .find(|(_, render)| render.image == image)
                .map(|(entity, _)| *entity);
            for (entity, _) in existing {
                if Some(entity) != keep {
                    commands.entity(entity).despawn();
                }
            }
            if keep.is_none() {
                let alpha = transition_alpha(background.transition.clone());
                let mut entity = commands.spawn((
                    BackgroundRender {
                        image: image.clone(),
                    },
                    Sprite::from_image(load_image(asset_server, resolver, |resolver| {
                        resolver.0.background(&image)
                    })),
                    Transform::from_xyz(0.0, 0.0, -10.0),
                ));
                if let Some(alpha) = alpha {
                    entity.insert(alpha);
                }
            }
        }
        None => {
            for (entity, _) in presentation.background_renders.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
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
        let existing = presentation
            .sprite_renders
            .iter()
            .find(|(_, render)| render.tag == sprite.tag);
        let unchanged = existing.is_some_and(|(_, render)| {
            render.attrs == sprite.attrs && render.position == sprite.position
        });
        if unchanged {
            continue;
        }
        if let Some((entity, _)) = existing {
            commands.entity(entity).despawn();
        }
        let alpha = transition_alpha(sprite.transition.clone());
        let mut entity = commands.spawn((
            SpriteRender {
                tag: sprite.tag.clone(),
                attrs: sprite.attrs.clone(),
                position: sprite.position.clone(),
            },
            Sprite::from_image(load_image(asset_server, resolver, |resolver| {
                resolver.0.sprite(&sprite.tag, &sprite.attrs)
            })),
            Transform::from_translation(position_to_translation(&sprite.position)),
        ));
        if let Some(alpha) = alpha {
            entity.insert(alpha);
        }
    }
}

fn load_image(
    asset_server: &Option<Res<AssetServer>>,
    resolver: &Option<Res<VnAssetResolver>>,
    path: impl FnOnce(&VnAssetResolver) -> std::path::PathBuf,
) -> Handle<Image> {
    match (asset_server, resolver) {
        (Some(asset_server), Some(resolver)) => {
            asset_server.load(path(resolver).to_string_lossy().to_string())
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
            asset_server.load(path(resolver).to_string_lossy().to_string())
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

fn transition_alpha(transition: Option<vn_core::Transition>) -> Option<TransitionAlpha> {
    let duration_ms = transition?.duration_ms;
    (duration_ms > 0).then_some(TransitionAlpha {
        elapsed_ms: 0,
        duration_ms,
    })
}

fn position_to_translation(position: &str) -> Vec3 {
    match position {
        "left" => Vec3::new(-320.0, -80.0, 0.0),
        "right" => Vec3::new(320.0, -80.0, 0.0),
        _ => Vec3::new(0.0, -80.0, 0.0),
    }
}
