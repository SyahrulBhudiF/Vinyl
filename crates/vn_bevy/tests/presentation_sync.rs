use bevy::prelude::*;
use std::time::Duration;
use vn_bevy::{
    BackgroundRender, MusicRender, PendingChoice, PresentationBackground, PresentationCommandQueue,
    PresentationDialogue, PresentationMenu, PresentationMusic, PresentationSprite, SpriteRender,
    TextReveal, TransitionAlpha, VnAssetResolver, VnBevyPlugin, VnRenderable, VnStory,
};
use vn_core::{Choice, Script, SourcePos, Stmt, StmtKind, TextEffect, Transition, compile};
use vn_runtime::PresentationCommand;

fn app_with_plugin() -> App {
    let mut app = App::new();
    app.add_plugins(VnBevyPlugin);
    app
}

fn push(app: &mut App, command: PresentationCommand) {
    app.world_mut()
        .resource_mut::<PresentationCommandQueue>()
        .push(command);
}

fn collect_backgrounds(app: &mut App) -> Vec<String> {
    app.world_mut()
        .query::<&PresentationBackground>()
        .iter(app.world())
        .map(|background| background.image.clone())
        .collect()
}

fn collect_transition_alphas(app: &mut App) -> Vec<u32> {
    app.world_mut()
        .query::<&TransitionAlpha>()
        .iter(app.world())
        .map(TransitionAlpha::alpha_permille)
        .collect()
}

fn collect_text_reveals(app: &mut App) -> Vec<usize> {
    app.world_mut()
        .query::<&TextReveal>()
        .iter(app.world())
        .map(TextReveal::visible_chars)
        .collect()
}

fn collect_sprites(app: &mut App) -> Vec<(String, Vec<String>, String)> {
    app.world_mut()
        .query::<&PresentationSprite>()
        .iter(app.world())
        .map(|sprite| {
            (
                sprite.tag.clone(),
                sprite.attrs.clone(),
                sprite.position.clone(),
            )
        })
        .collect()
}

fn collect_dialogues(app: &mut App) -> Vec<(Option<String>, String)> {
    app.world_mut()
        .query::<&PresentationDialogue>()
        .iter(app.world())
        .map(|dialogue| (dialogue.speaker.clone(), dialogue.text.clone()))
        .collect()
}

fn collect_music(app: &mut App) -> Vec<String> {
    app.world_mut()
        .query::<&PresentationMusic>()
        .iter(app.world())
        .map(|music| music.path.clone())
        .collect()
}

fn collect_menus(app: &mut App) -> Vec<Vec<String>> {
    app.world_mut()
        .query::<&PresentationMenu>()
        .iter(app.world())
        .map(|menu| menu.choices.clone())
        .collect()
}

fn pos() -> SourcePos {
    SourcePos {
        file: "bevy_test.vn".to_string(),
        line: 1,
        column: 1,
    }
}

fn stmt(kind: StmtKind) -> Stmt {
    Stmt { kind, pos: pos() }
}

fn collect_background_renders(app: &mut App) -> Vec<Vec3> {
    app.world_mut()
        .query_filtered::<&Transform, With<BackgroundRender>>()
        .iter(app.world())
        .map(|transform| transform.translation)
        .collect()
}

fn collect_music_renders(app: &mut App) -> Vec<String> {
    app.world_mut()
        .query::<&MusicRender>()
        .iter(app.world())
        .map(|music| music.path.clone())
        .collect()
}

fn collect_sprite_renders(app: &mut App) -> Vec<(String, Vec3)> {
    let mut renders = app
        .world_mut()
        .query::<(&SpriteRender, &Transform)>()
        .iter(app.world())
        .map(|(render, transform)| (render.tag.clone(), transform.translation))
        .collect::<Vec<_>>();
    renders.sort_by(|left, right| left.0.cmp(&right.0));
    renders
}

#[test]
fn transition_and_text_timers_tick_and_complete_deterministically() {
    let mut app = app_with_plugin();
    app.insert_resource(VnRenderable(true));
    app.insert_resource(VnAssetResolver::new("."));

    push(
        &mut app,
        PresentationCommand::SetBackground {
            image: "bg room".to_string(),
            transition: Some(Transition {
                kind: "fade".to_string(),
                duration_ms: 1000,
            }),
        },
    );
    push(
        &mut app,
        PresentationCommand::ShowDialogue {
            speaker: None,
            text: "HelloWorld".to_string(),
            effect: TextEffect::Typewriter {
                chars_per_second: 10,
            },
        },
    );
    app.update();

    assert_eq!(collect_transition_alphas(&mut app), vec![0, 0]);
    assert_eq!(collect_text_reveals(&mut app), vec![0]);

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_millis(500));
    app.update();

    assert_eq!(collect_transition_alphas(&mut app), vec![500, 500]);
    assert_eq!(collect_text_reveals(&mut app), vec![5]);

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_millis(500));
    app.update();

    assert!(collect_transition_alphas(&mut app).is_empty());
    assert!(collect_text_reveals(&mut app).is_empty());
}

#[test]
fn instant_commands_do_not_create_transition_state() {
    let mut app = app_with_plugin();
    app.insert_resource(VnRenderable(true));
    app.insert_resource(VnAssetResolver::new("."));

    push(
        &mut app,
        PresentationCommand::SetBackground {
            image: "bg room".to_string(),
            transition: None,
        },
    );
    push(
        &mut app,
        PresentationCommand::ShowSprite {
            tag: "eileen".to_string(),
            attrs: vec!["happy".to_string()],
            position: "center".to_string(),
            transition: None,
        },
    );
    push(
        &mut app,
        PresentationCommand::ShowDialogue {
            speaker: None,
            text: "Hello".to_string(),
            effect: TextEffect::Instant,
        },
    );
    app.update();

    assert!(collect_transition_alphas(&mut app).is_empty());
    assert!(collect_text_reveals(&mut app).is_empty());
}

fn choice_story() -> VnStory {
    VnStory::new(compile(&Script {
        statements: vec![
            stmt(StmtKind::Menu {
                choices: vec![
                    Choice {
                        text_id: None,
                        text: "Stay".to_string(),
                        condition: None,
                        body: vec![stmt(StmtKind::Say {
                            speaker: Some("Guide".to_string()),
                            text_id: None,
                            text: "You stayed.".to_string(),
                            effect: vn_core::TextEffect::Instant,
                        })],
                        pos: pos(),
                    },
                    Choice {
                        text_id: None,
                        text: "Leave".to_string(),
                        condition: None,
                        body: vec![
                            stmt(StmtKind::Scene {
                                image: "bg hallway".to_string(),
                                transition: None,
                            }),
                            stmt(StmtKind::Show {
                                tag: "eileen".to_string(),
                                attrs: vec!["concerned".to_string()],
                                position: "right".to_string(),
                                transition: None,
                            }),
                            stmt(StmtKind::Say {
                                speaker: Some("Eileen".to_string()),
                                text_id: None,
                                text: "We should leave now.".to_string(),
                                effect: vn_core::TextEffect::Instant,
                            }),
                        ],
                        pos: pos(),
                    },
                ],
            }),
            stmt(StmtKind::End),
        ],
    }))
}

fn dialogue_story() -> VnStory {
    VnStory::new(compile(&Script {
        statements: vec![stmt(StmtKind::Say {
            speaker: Some("Narrator".to_string()),
            text_id: None,
            text: "Space advances the story.".to_string(),
            effect: vn_core::TextEffect::Instant,
        })],
    }))
}

fn dialogue_then_visuals_then_dialogue_story() -> VnStory {
    VnStory::new(compile(&Script {
        statements: vec![
            stmt(StmtKind::Say {
                speaker: Some("Narrator".to_string()),
                text_id: None,
                text: "The room is quiet.".to_string(),
                effect: vn_core::TextEffect::Instant,
            }),
            stmt(StmtKind::Scene {
                image: "bg hallway".to_string(),
                transition: None,
            }),
            stmt(StmtKind::Show {
                tag: "eileen".to_string(),
                attrs: vec!["worried".to_string()],
                position: "left".to_string(),
                transition: None,
            }),
            stmt(StmtKind::Hide {
                tag: "eileen".to_string(),
            }),
            stmt(StmtKind::PlayMusic {
                path: "audio/tension.ogg".to_string(),
            }),
            stmt(StmtKind::StopMusic),
            stmt(StmtKind::Say {
                speaker: Some("Eileen".to_string()),
                text_id: None,
                text: "We should move before anyone sees us.".to_string(),
                effect: vn_core::TextEffect::Instant,
            }),
        ],
    }))
}

fn menu_only_story() -> VnStory {
    VnStory::new(compile(&Script {
        statements: vec![stmt(StmtKind::Menu {
            choices: vec![Choice {
                text_id: None,
                text: "Continue".to_string(),
                condition: None,
                body: vec![stmt(StmtKind::Say {
                    speaker: None,
                    text_id: None,
                    text: "Chosen.".to_string(),
                    effect: vn_core::TextEffect::Instant,
                })],
                pos: pos(),
            }],
        })],
    }))
}
#[test]
fn queued_commands_materialize_presentation_marker_entities() {
    let mut app = app_with_plugin();

    push(
        &mut app,
        PresentationCommand::SetBackground {
            image: "bg classroom".to_string(),
            transition: None,
        },
    );
    push(
        &mut app,
        PresentationCommand::ShowSprite {
            tag: "eileen".to_string(),
            attrs: vec!["happy".to_string(), "uniform".to_string()],
            position: "left".to_string(),
            transition: None,
        },
    );
    push(
        &mut app,
        PresentationCommand::ShowDialogue {
            speaker: Some("Eileen".to_string()),
            text: "The sync system should expose this line.".to_string(),
            effect: vn_core::TextEffect::Instant,
        },
    );
    push(
        &mut app,
        PresentationCommand::ShowMenu(vec!["Stay".to_string(), "Leave".to_string()]),
    );
    push(
        &mut app,
        PresentationCommand::PlayMusic("bgm/theme.ogg".to_string()),
    );

    app.update();

    assert_eq!(
        collect_backgrounds(&mut app),
        vec!["bg classroom".to_string()]
    );
    assert_eq!(
        collect_sprites(&mut app),
        vec![(
            "eileen".to_string(),
            vec!["happy".to_string(), "uniform".to_string()],
            "left".to_string(),
        )]
    );
    assert_eq!(
        collect_dialogues(&mut app),
        vec![(
            Some("Eileen".to_string()),
            "The sync system should expose this line.".to_string(),
        )]
    );
    assert_eq!(
        collect_menus(&mut app),
        vec![vec!["Stay".to_string(), "Leave".to_string()]]
    );
    assert_eq!(collect_music(&mut app), vec!["bgm/theme.ogg".to_string()]);
    assert!(
        app.world()
            .resource::<PresentationCommandQueue>()
            .is_empty()
    );
}

#[test]
fn later_updates_replace_removed_presentation_entities() {
    let mut app = app_with_plugin();

    push(
        &mut app,
        PresentationCommand::SetBackground {
            image: "bg room".to_string(),
            transition: None,
        },
    );
    push(
        &mut app,
        PresentationCommand::ShowSprite {
            tag: "eileen".to_string(),
            attrs: vec!["happy".to_string()],
            position: "center".to_string(),
            transition: None,
        },
    );
    push(
        &mut app,
        PresentationCommand::ShowSprite {
            tag: "lucy".to_string(),
            attrs: vec!["neutral".to_string()],
            position: "right".to_string(),
            transition: None,
        },
    );
    push(
        &mut app,
        PresentationCommand::ShowMenu(vec!["Continue".to_string()]),
    );
    app.update();

    push(
        &mut app,
        PresentationCommand::SetBackground {
            image: "bg hallway".to_string(),
            transition: None,
        },
    );
    push(
        &mut app,
        PresentationCommand::ShowSprite {
            tag: "lucy".to_string(),
            attrs: vec!["surprised".to_string()],
            position: "left".to_string(),
            transition: None,
        },
    );
    push(&mut app, PresentationCommand::ClearMenu);
    app.update();

    assert_eq!(
        collect_backgrounds(&mut app),
        vec!["bg hallway".to_string()]
    );
    assert_eq!(
        collect_sprites(&mut app),
        vec![(
            "lucy".to_string(),
            vec!["surprised".to_string()],
            "left".to_string(),
        ),]
    );
    assert!(collect_menus(&mut app).is_empty());
}

#[test]
fn renderable_false_keeps_presentation_markers_headless() {
    let mut app = app_with_plugin();
    app.insert_resource(VnAssetResolver::new("."));

    push(
        &mut app,
        PresentationCommand::SetBackground {
            image: "classroom".to_string(),
            transition: None,
        },
    );
    push(
        &mut app,
        PresentationCommand::ShowSprite {
            tag: "eileen".to_string(),
            attrs: vec!["happy".to_string()],
            position: "left".to_string(),
            transition: None,
        },
    );
    app.update();

    assert_eq!(collect_backgrounds(&mut app), vec!["classroom".to_string()]);
    assert_eq!(
        collect_sprites(&mut app),
        vec![(
            "eileen".to_string(),
            vec!["happy".to_string()],
            "left".to_string(),
        )]
    );
    assert!(collect_background_renders(&mut app).is_empty());
    assert!(collect_sprite_renders(&mut app).is_empty());
}

#[test]
fn music_commands_create_replace_and_stop_music_entities() {
    let mut app = app_with_plugin();
    app.insert_resource(VnRenderable(true));
    app.insert_resource(VnAssetResolver::new("."));

    push(
        &mut app,
        PresentationCommand::PlayMusic("bgm/first.ogg".to_string()),
    );
    app.update();
    assert_eq!(collect_music(&mut app), vec!["bgm/first.ogg".to_string()]);
    assert_eq!(
        collect_music_renders(&mut app),
        vec!["bgm/first.ogg".to_string()]
    );

    push(
        &mut app,
        PresentationCommand::PlayMusic("bgm/second.ogg".to_string()),
    );
    app.update();
    assert_eq!(collect_music(&mut app), vec!["bgm/second.ogg".to_string()]);
    assert_eq!(
        collect_music_renders(&mut app),
        vec!["bgm/second.ogg".to_string()]
    );

    push(&mut app, PresentationCommand::StopMusic);
    app.update();
    assert!(collect_music(&mut app).is_empty());
    assert!(collect_music_renders(&mut app).is_empty());
}

#[test]
fn renderable_true_materializes_render_entities_from_presentation_markers() {
    let mut app = app_with_plugin();
    app.insert_resource(VnRenderable(true));
    app.insert_resource(VnAssetResolver::new("."));

    push(
        &mut app,
        PresentationCommand::SetBackground {
            image: "classroom".to_string(),
            transition: None,
        },
    );
    push(
        &mut app,
        PresentationCommand::ShowSprite {
            tag: "eileen".to_string(),
            attrs: vec!["happy".to_string()],
            position: "left".to_string(),
            transition: None,
        },
    );
    push(
        &mut app,
        PresentationCommand::ShowSprite {
            tag: "lucy".to_string(),
            attrs: vec!["neutral".to_string()],
            position: "right".to_string(),
            transition: None,
        },
    );
    app.update();

    assert_eq!(
        collect_background_renders(&mut app),
        vec![Vec3::new(0.0, 0.0, -10.0)]
    );
    assert_eq!(
        collect_sprite_renders(&mut app),
        vec![
            ("eileen".to_string(), Vec3::new(-320.0, -80.0, 0.0)),
            ("lucy".to_string(), Vec3::new(320.0, -80.0, 0.0)),
        ]
    );
}

#[test]
fn pending_choice_selects_menu_branch_and_applies_queued_presentation_state() {
    let mut app = app_with_plugin();
    let mut story = choice_story();
    story.continue_story().unwrap();
    app.insert_resource(story);
    app.insert_resource(PendingChoice(1));

    app.update();
    app.update();
    app.update();

    assert!(!app.world().contains_resource::<PendingChoice>());
    assert!(collect_menus(&mut app).is_empty());
    assert_eq!(
        collect_backgrounds(&mut app),
        vec!["bg hallway".to_string()]
    );
    assert_eq!(
        collect_sprites(&mut app),
        vec![(
            "eileen".to_string(),
            vec!["concerned".to_string()],
            "right".to_string(),
        )]
    );
    assert_eq!(
        collect_dialogues(&mut app),
        vec![(
            Some("Eileen".to_string()),
            "We should leave now.".to_string(),
        )]
    );
}

#[test]
fn keyboard_advance_queues_and_applies_next_dialogue() {
    let mut app = app_with_plugin();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.insert_resource(dialogue_story());
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Space);

    app.update();

    assert_eq!(
        collect_dialogues(&mut app),
        vec![(
            Some("Narrator".to_string()),
            "Space advances the story.".to_string(),
        )]
    );
    assert!(
        app.world()
            .resource::<PresentationCommandQueue>()
            .is_empty()
    );
}

#[test]
fn keyboard_advance_drains_visual_events_to_next_dialogue() {
    let mut app = app_with_plugin();
    app.init_resource::<ButtonInput<KeyCode>>();
    let mut story = dialogue_then_visuals_then_dialogue_story();
    story.continue_story().unwrap();
    app.insert_resource(story);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Space);

    app.update();

    assert_eq!(
        collect_backgrounds(&mut app),
        vec!["bg hallway".to_string()]
    );
    assert!(collect_sprites(&mut app).is_empty());
    assert_eq!(
        collect_dialogues(&mut app),
        vec![(
            Some("Eileen".to_string()),
            "We should move before anyone sees us.".to_string(),
        )]
    );
    assert!(
        app.world()
            .resource::<PresentationCommandQueue>()
            .is_empty()
    );
}

#[test]
fn keyboard_advance_does_not_choose_while_story_is_waiting_on_menu() {
    let mut app = app_with_plugin();
    app.init_resource::<ButtonInput<KeyCode>>();
    let mut story = menu_only_story();
    story.continue_story().unwrap();
    app.insert_resource(story);
    push(
        &mut app,
        PresentationCommand::ShowMenu(vec!["Continue".to_string()]),
    );
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Enter);

    app.update();

    assert_eq!(collect_menus(&mut app), vec![vec!["Continue".to_string()]]);
    assert!(collect_dialogues(&mut app).is_empty());
}
