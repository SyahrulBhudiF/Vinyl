use crate::components::{
    PresentationBackground, PresentationDialogue, PresentationMenu, PresentationMusic,
    PresentationSprite,
};
use crate::resources::{PresentationCommandQueue, VnPresentation};
use bevy::prelude::*;
use vn_runtime::apply_command;

pub(crate) fn apply_queued_commands(
    mut queue: ResMut<PresentationCommandQueue>,
    mut presentation: ResMut<VnPresentation>,
) {
    while let Some(command) = queue.commands.pop_front() {
        apply_command(&mut presentation.0, &command);
    }
}

pub(crate) fn sync_presentation_entities(
    mut commands: Commands,
    presentation: Res<VnPresentation>,
    backgrounds: Query<Entity, With<PresentationBackground>>,
    sprites: Query<Entity, With<PresentationSprite>>,
    dialogues: Query<Entity, With<PresentationDialogue>>,
    menus: Query<Entity, With<PresentationMenu>>,
    music: Query<Entity, With<PresentationMusic>>,
) {
    despawn_all(&mut commands, backgrounds.iter());
    despawn_all(&mut commands, sprites.iter());
    despawn_all(&mut commands, dialogues.iter());
    despawn_all(&mut commands, menus.iter());
    despawn_all(&mut commands, music.iter());

    if let Some(image) = &presentation.0.background {
        commands.spawn(PresentationBackground {
            image: image.clone(),
        });
    }

    let mut sprites = presentation.0.sprites.iter().collect::<Vec<_>>();
    sprites.sort_by(|left, right| left.0.cmp(right.0));
    for (tag, sprite) in sprites {
        commands.spawn(PresentationSprite {
            tag: tag.clone(),
            attrs: sprite.attrs.clone(),
            position: sprite.position.clone(),
        });
    }

    if let Some(dialogue) = &presentation.0.dialogue {
        commands.spawn(PresentationDialogue {
            speaker: dialogue.speaker.clone(),
            text: dialogue.text.clone(),
        });
    }

    if let Some(menu) = &presentation.0.menu {
        commands.spawn(PresentationMenu {
            choices: menu.clone(),
        });
    }

    if let Some(path) = &presentation.0.music {
        commands.spawn(PresentationMusic { path: path.clone() });
    }
}

fn despawn_all(commands: &mut Commands, entities: impl Iterator<Item = Entity>) {
    for entity in entities {
        commands.entity(entity).despawn();
    }
}
