use vn_core::{PresentationSnapshot, SpriteSnapshot, TextEffect, VmEvent};
use vn_runtime::{PresentationCommand, apply_command, commands_from_event};

#[test]
fn dialogue_event_clears_menu_and_sets_dialogue() {
    let event = VmEvent::Dialogue {
        speaker: Some("eileen".to_string()),
        text: "Hello.".to_string(),
        effect: TextEffect::Instant,
    };
    let commands = commands_from_event(&event);
    assert_eq!(
        commands,
        vec![
            PresentationCommand::ClearMenu,
            PresentationCommand::ShowDialogue {
                speaker: Some("eileen".to_string()),
                text: "Hello.".to_string(),
                effect: TextEffect::Instant,
            },
        ]
    );

    let mut snapshot = PresentationSnapshot {
        menu: Some(vec!["Continue".to_string()]),
        ..Default::default()
    };
    for command in &commands {
        apply_command(&mut snapshot, command);
    }
    assert_eq!(snapshot.menu, None);
    assert_eq!(snapshot.dialogue.unwrap().text, "Hello.");
}

#[test]
fn scene_command_resets_sprites() {
    let mut snapshot = PresentationSnapshot::default();
    apply_command(
        &mut snapshot,
        &PresentationCommand::ShowSprite {
            tag: "eileen".to_string(),
            attrs: vec!["happy".to_string()],
            position: "center".to_string(),
            transition: None,
        },
    );
    apply_command(
        &mut snapshot,
        &PresentationCommand::SetBackground {
            image: "bg room".to_string(),
            transition: None,
        },
    );

    assert_eq!(snapshot.background.as_deref(), Some("bg room"));
    assert!(snapshot.sprites.is_empty());
}

#[test]
fn sprite_and_audio_commands_update_only_their_targeted_snapshot_fields() {
    let mut snapshot = PresentationSnapshot {
        background: Some("bg plaza".to_string()),
        music: Some("assets/audio/bgm/old.ogg".to_string()),
        menu: Some(vec!["Ask".to_string()]),
        ..Default::default()
    };

    apply_command(
        &mut snapshot,
        &PresentationCommand::ShowSprite {
            tag: "eileen".to_string(),
            attrs: vec!["happy".to_string(), "casual".to_string()],
            position: "left".to_string(),
            transition: None,
        },
    );
    apply_command(
        &mut snapshot,
        &PresentationCommand::ShowSprite {
            tag: "bob".to_string(),
            attrs: vec!["neutral".to_string()],
            position: "right".to_string(),
            transition: None,
        },
    );
    apply_command(
        &mut snapshot,
        &PresentationCommand::PlayMusic("assets/audio/bgm/new.ogg".to_string()),
    );

    assert_eq!(snapshot.background.as_deref(), Some("bg plaza"));
    assert_eq!(snapshot.menu.as_deref(), Some(&["Ask".to_string()][..]));
    assert_eq!(snapshot.music.as_deref(), Some("assets/audio/bgm/new.ogg"));
    assert_eq!(
        snapshot.sprites.get("eileen"),
        Some(&SpriteSnapshot {
            attrs: vec!["happy".to_string(), "casual".to_string()],
            position: "left".to_string(),
        })
    );
    assert_eq!(
        snapshot.sprites.get("bob"),
        Some(&SpriteSnapshot {
            attrs: vec!["neutral".to_string()],
            position: "right".to_string(),
        })
    );

    apply_command(
        &mut snapshot,
        &PresentationCommand::HideSprite("eileen".to_string()),
    );
    apply_command(&mut snapshot, &PresentationCommand::StopMusic);

    assert!(!snapshot.sprites.contains_key("eileen"));
    assert!(snapshot.sprites.contains_key("bob"));
    assert_eq!(snapshot.music, None);
}

#[test]
fn event_commands_apply_to_the_same_snapshot_state_as_the_vm_events_describe() {
    let events = [
        VmEvent::Scene {
            image: "bg classroom".to_string(),
            transition: None,
        },
        VmEvent::Show {
            tag: "eileen".to_string(),
            attrs: vec!["happy".to_string()],
            position: "center".to_string(),
            transition: None,
        },
        VmEvent::PlayMusic {
            path: "assets/audio/bgm/theme.ogg".to_string(),
        },
        VmEvent::Menu {
            choices: vec!["Stay".to_string(), "Leave".to_string()],
        },
        VmEvent::Hide {
            tag: "eileen".to_string(),
        },
        VmEvent::StopMusic,
    ];
    let mut snapshot = PresentationSnapshot::default();

    for event in &events {
        for command in commands_from_event(event) {
            apply_command(&mut snapshot, &command);
        }
    }

    assert_eq!(snapshot.background.as_deref(), Some("bg classroom"));
    assert!(snapshot.sprites.is_empty());
    assert_eq!(snapshot.music, None);
    assert_eq!(
        snapshot.menu,
        Some(vec!["Stay".to_string(), "Leave".to_string()])
    );
}
