use vn_core::{DialogueSnapshot, PresentationSnapshot, SpriteSnapshot, VmEvent};

/// Renderer-independent command emitted from VM events.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PresentationCommand {
    SetBackground(String),
    ShowSprite {
        tag: String,
        attrs: Vec<String>,
        position: String,
    },
    HideSprite(String),
    PlayMusic(String),
    StopMusic,
    ShowDialogue {
        speaker: Option<String>,
        text: String,
    },
    ShowMenu(Vec<String>),
    ClearMenu,
}

/// Converts a VM event to presentation commands.
pub fn commands_from_event(event: &VmEvent) -> Vec<PresentationCommand> {
    match event {
        VmEvent::Dialogue { speaker, text } => vec![
            PresentationCommand::ClearMenu,
            PresentationCommand::ShowDialogue {
                speaker: speaker.clone(),
                text: text.clone(),
            },
        ],
        VmEvent::Scene { image } => vec![PresentationCommand::SetBackground(image.clone())],
        VmEvent::Show {
            tag,
            attrs,
            position,
        } => vec![PresentationCommand::ShowSprite {
            tag: tag.clone(),
            attrs: attrs.clone(),
            position: position.clone(),
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
        PresentationCommand::SetBackground(image) => {
            snapshot.background = Some(image.clone());
            snapshot.sprites.clear();
        }
        PresentationCommand::ShowSprite {
            tag,
            attrs,
            position,
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
        PresentationCommand::ShowDialogue { speaker, text } => {
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
