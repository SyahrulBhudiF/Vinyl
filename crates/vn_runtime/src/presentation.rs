use vn_core::{
    DialogueSnapshot, PresentationSnapshot, SpriteSnapshot, TextEffect, Transition, VmEvent,
};

/// Renderer-independent command emitted from VM events.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PresentationCommand {
    SetBackground {
        image: String,
        transition: Option<Transition>,
    },
    ShowSprite {
        tag: String,
        attrs: Vec<String>,
        position: String,
        transition: Option<Transition>,
    },
    HideSprite(String),
    PlayMusic(String),
    StopMusic,
    ShowDialogue {
        speaker: Option<String>,
        text: String,
        effect: TextEffect,
    },
    ShowMenu(Vec<String>),
    ClearMenu,
}

/// Converts a VM event to presentation commands.
pub fn commands_from_event(event: &VmEvent) -> Vec<PresentationCommand> {
    match event {
        VmEvent::Dialogue {
            speaker,
            text,
            effect,
        } => vec![
            PresentationCommand::ClearMenu,
            PresentationCommand::ShowDialogue {
                speaker: speaker.clone(),
                text: text.clone(),
                effect: effect.clone(),
            },
        ],
        VmEvent::Scene { image, transition } => vec![PresentationCommand::SetBackground {
            image: image.clone(),
            transition: transition.clone(),
        }],
        VmEvent::Show {
            tag,
            attrs,
            position,
            transition,
        } => vec![PresentationCommand::ShowSprite {
            tag: tag.clone(),
            attrs: attrs.clone(),
            position: position.clone(),
            transition: transition.clone(),
        }],
        VmEvent::Hide { tag } => vec![PresentationCommand::HideSprite(tag.clone())],
        VmEvent::PlayMusic { path } => vec![PresentationCommand::PlayMusic(path.clone())],
        VmEvent::StopMusic => vec![PresentationCommand::StopMusic],
        VmEvent::Menu { choices } => vec![PresentationCommand::ShowMenu(choices.clone())],
        VmEvent::End => vec![PresentationCommand::ClearMenu],
    }
}

/// Applies a presentation command to a serializable snapshot.
pub fn apply_command(snapshot: &mut PresentationSnapshot, command: &PresentationCommand) {
    match command {
        PresentationCommand::SetBackground { image, .. } => {
            snapshot.background = Some(image.clone());
            snapshot.sprites.clear();
        }
        PresentationCommand::ShowSprite {
            tag,
            attrs,
            position,
            ..
        } => {
            snapshot.sprites.insert(
                tag.clone(),
                SpriteSnapshot {
                    attrs: attrs.clone(),
                    position: position.clone(),
                },
            );
        }
        PresentationCommand::HideSprite(tag) => {
            snapshot.sprites.remove(tag);
        }
        PresentationCommand::PlayMusic(path) => {
            snapshot.music = Some(path.clone());
        }
        PresentationCommand::StopMusic => {
            snapshot.music = None;
        }
        PresentationCommand::ShowDialogue { speaker, text, .. } => {
            snapshot.dialogue = Some(DialogueSnapshot {
                speaker: speaker.clone(),
                text: text.clone(),
            });
        }
        PresentationCommand::ShowMenu(choices) => {
            snapshot.menu = Some(choices.clone());
        }
        PresentationCommand::ClearMenu => {
            snapshot.menu = None;
        }
    }
}
