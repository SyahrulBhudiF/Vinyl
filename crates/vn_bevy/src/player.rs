use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy::winit::{UpdateMode, WinitSettings};
use vn_core::{Program, ProjectId, VmError, VmEvent};
use vn_script::ProjectManifest;

use crate::{
    AssetLoadingState, MenuFocus, PendingChoice, PresentationCommandQueue, VnAssetResolver,
    VnBevyPlugin, VnRenderable, VnStory,
    components::{PresentationDialogue, PresentationMenu, TextReveal, TransitionAlpha},
    input::queue_event_and_following_visuals,
};

/// Data required to start the desktop player after CLI validation.
pub struct PlayerConfig {
    pub project_root: PathBuf,
    pub manifest: ProjectManifest,
    pub program: Program,
    pub translations: HashMap<String, String>,
    pub project_id: ProjectId,
    pub script_hash: String,
    pub engine_version: String,
}

/// Current top-level player screen/mode.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub enum PlayerMode {
    #[default]
    Boot,
    Loading,
    Playing,
    Paused,
    Save,
    Load,
    Settings,
    RuntimeError,
    Ended,
}

/// Dense player runtime flags.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct PlayerFlags(u8);

impl PlayerFlags {
    const PAUSED: u8 = 1 << 0;
    const ENDED: u8 = 1 << 1;
    const MUTED: u8 = 1 << 2;
    const FULLSCREEN: u8 = 1 << 3;
    const AUTO_ADVANCE: u8 = 1 << 4;
    const LOADING: u8 = 1 << 5;
    const TRANSITION_ACTIVE: u8 = 1 << 6;

    pub fn paused(self) -> bool {
        self.contains(Self::PAUSED)
    }

    pub fn ended(self) -> bool {
        self.contains(Self::ENDED)
    }

    pub fn muted(self) -> bool {
        self.contains(Self::MUTED)
    }

    pub fn fullscreen(self) -> bool {
        self.contains(Self::FULLSCREEN)
    }

    pub fn auto_advance(self) -> bool {
        self.contains(Self::AUTO_ADVANCE)
    }

    pub fn loading(self) -> bool {
        self.contains(Self::LOADING)
    }

    pub fn transition_active(self) -> bool {
        self.contains(Self::TRANSITION_ACTIVE)
    }

    fn contains(self, flag: u8) -> bool {
        self.0 & flag != 0
    }

    fn set(&mut self, flag: u8, enabled: bool) {
        if enabled {
            self.0 |= flag;
        } else {
            self.0 &= !flag;
        }
    }
}

/// Validates VM construction, then runs the reusable desktop Bevy player.
pub fn run_player(config: PlayerConfig) -> Result<(), VmError> {
    let story = VnStory::with_translations(config.program, config.translations)?;
    let asset_root = config
        .project_root
        .canonicalize()
        .unwrap_or_else(|_| config.project_root.clone())
        .to_string_lossy()
        .into_owned();
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.06)))
        .insert_resource(PlayerMode::Boot)
        .insert_resource(PlayerFlags::default())
        .insert_resource(story)
        .insert_resource(VnAssetResolver::with_manifest(
            config.project_root,
            config.manifest,
        ))
        .insert_resource(VnRenderable(true))
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::reactive(Duration::from_secs_f64(1.0 / 60.0)),
            unfocused_mode: UpdateMode::reactive_low_power(Duration::from_secs(1)),
        })
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: format!("Vinyl — {}", config.project_id.0),
                        resolution: WindowResolution::new(1280, 720),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(VnBevyPlugin)
        .add_systems(Startup, (install_bundled_font, start_story).chain())
        .add_systems(
            Update,
            (
                sync_player_state,
                sync_story_ui,
                update_dialogue_text,
                menu_button_input,
                runtime_error_quit,
            )
                .chain(),
        )
        .run();
    Ok(())
}

#[derive(Component)]
struct PlayerOverlay;

#[derive(Component)]
struct RuntimeErrorQuit;

#[derive(Component)]
struct StoryUi {
    dialogue: String,
    choices: Vec<String>,
}

#[derive(Component)]
struct DialogueText;

#[derive(Component)]
struct ChoiceButton(usize);

#[derive(Resource)]
struct PlayerFont(Handle<Font>);

fn install_bundled_font(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    let font = Font::try_from_bytes(include_bytes!("../assets/fonts/NotoSans.ttf").to_vec())
        .expect("bundled Noto Sans font is valid");
    commands.insert_resource(PlayerFont(fonts.add(font)));
}

fn sync_story_ui(
    mut commands: Commands,
    font: Res<PlayerFont>,
    dialogue: Query<&PresentationDialogue>,
    menu: Query<&PresentationMenu>,
    existing: Query<(Entity, &StoryUi)>,
    focus: Res<MenuFocus>,
) {
    let dialogue = dialogue.iter().next();
    let choices = menu
        .iter()
        .next()
        .map(|menu| menu.choices.clone())
        .unwrap_or_default();
    let Some(dialogue) = dialogue else {
        for (entity, _) in &existing {
            commands.entity(entity).despawn();
        }
        return;
    };
    let dialogue_key = format!("{:?}\n{}", dialogue.speaker, dialogue.text);
    if !focus.is_changed()
        && existing
            .iter()
            .any(|(_, ui)| ui.dialogue == dialogue_key && ui.choices == choices)
    {
        return;
    }
    for (entity, _) in &existing {
        commands.entity(entity).despawn();
    }
    let font = font.0.clone();
    commands
        .spawn((
            StoryUi {
                dialogue: dialogue_key,
                choices: choices.clone(),
            },
            Node {
                position_type: PositionType::Absolute,
                left: px(64),
                right: px(64),
                bottom: px(42),
                padding: UiRect::axes(px(30), px(22)),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.03, 0.05, 0.92)),
            BorderRadius::all(px(12)),
        ))
        .with_children(|panel| {
            if let Some(speaker) = &dialogue.speaker {
                panel.spawn((
                    Text::new(speaker.clone()),
                    TextFont {
                        font: font.clone(),
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.55, 0.78, 1.0)),
                ));
            }
            panel.spawn((
                DialogueText,
                Text::new(dialogue.text.clone()),
                TextFont {
                    font: font.clone(),
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.96, 1.0)),
            ));
            for (index, choice) in choices.iter().enumerate() {
                panel
                    .spawn((
                        ChoiceButton(index),
                        Button,
                        Node {
                            width: percent(100),
                            padding: UiRect::axes(px(16), px(10)),
                            ..default()
                        },
                        BackgroundColor(if index == focus.0 {
                            Color::srgb(0.16, 0.25, 0.38)
                        } else {
                            Color::srgb(0.08, 0.11, 0.17)
                        }),
                        BorderRadius::all(px(6)),
                    ))
                    .with_child((
                        Text::new(format!("{}. {choice}", index + 1)),
                        TextFont {
                            font: font.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.88, 0.92, 1.0)),
                    ));
            }
        });
}

fn update_dialogue_text(
    dialogue: Query<(&PresentationDialogue, Option<&TextReveal>)>,
    mut text: Query<&mut Text, With<DialogueText>>,
) {
    let Some((dialogue, reveal)) = dialogue.iter().next() else {
        return;
    };
    let visible = reveal.map_or(dialogue.text.chars().count(), TextReveal::visible_chars);
    let shown = dialogue.text.chars().take(visible).collect::<String>();
    for mut text in &mut text {
        if text.0 != shown {
            text.0.clone_from(&shown);
        }
    }
}

fn menu_button_input(
    mut commands: Commands,
    mut buttons: Query<(&ChoiceButton, &Interaction, &mut BackgroundColor), Changed<Interaction>>,
    loading: Res<AssetLoadingState>,
    transitions: Query<(), With<TransitionAlpha>>,
) {
    if loading.started_at.is_some() || loading.error.is_some() || !transitions.is_empty() {
        return;
    }
    for (choice, interaction, mut color) in &mut buttons {
        match interaction {
            Interaction::Pressed => commands.insert_resource(PendingChoice(choice.0)),
            Interaction::Hovered => *color = Color::srgb(0.2, 0.32, 0.48).into(),
            Interaction::None => *color = Color::srgb(0.08, 0.11, 0.17).into(),
        }
    }
}

fn sync_player_state(
    mut commands: Commands,
    loading: Res<AssetLoadingState>,
    font: Res<PlayerFont>,
    overlays: Query<Entity, With<PlayerOverlay>>,
    mut mode: ResMut<PlayerMode>,
    mut flags: ResMut<PlayerFlags>,
) {
    let error = loading.error.as_deref();
    let show_loading = error.is_none() && loading.visible();
    flags.set(PlayerFlags::LOADING, loading.started_at.is_some());
    *mode = if error.is_some() {
        PlayerMode::RuntimeError
    } else if loading.started_at.is_some() {
        PlayerMode::Loading
    } else if flags.ended() {
        PlayerMode::Ended
    } else {
        PlayerMode::Playing
    };
    if !(show_loading || error.is_some()) {
        for entity in &overlays {
            commands.entity(entity).despawn();
        }
        return;
    }
    if !overlays.is_empty() {
        return;
    }
    let font = font.0.clone();
    commands
        .spawn((
            PlayerOverlay,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: px(20),
                ..default()
            },
            BackgroundColor(Color::srgba(0.015, 0.02, 0.035, 0.92)),
            GlobalZIndex(100),
        ))
        .with_children(|overlay| {
            overlay.spawn((
                Text::new(error.unwrap_or("Loading…")),
                TextFont {
                    font: font.clone(),
                    font_size: if error.is_some() { 24.0 } else { 32.0 },
                    ..default()
                },
                TextColor(if error.is_some() {
                    Color::srgb(1.0, 0.55, 0.55)
                } else {
                    Color::WHITE
                }),
                Node {
                    max_width: px(900),
                    ..default()
                },
            ));
            if error.is_some() {
                overlay
                    .spawn((
                        RuntimeErrorQuit,
                        Button,
                        Node {
                            padding: UiRect::axes(px(24), px(12)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.28, 0.08, 0.1)),
                        BorderRadius::all(px(6)),
                    ))
                    .with_child((
                        Text::new("Quit"),
                        TextFont {
                            font,
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
            }
        });
}

fn runtime_error_quit(
    buttons: Query<&Interaction, (Changed<Interaction>, With<RuntimeErrorQuit>)>,
    mut exit: MessageWriter<AppExit>,
) {
    if buttons
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        exit.write(AppExit::Success);
    }
}

fn start_story(
    mut story: ResMut<VnStory>,
    mut queue: ResMut<PresentationCommandQueue>,
    mut mode: ResMut<PlayerMode>,
    mut flags: ResMut<PlayerFlags>,
) {
    *mode = PlayerMode::Loading;
    flags.set(PlayerFlags::LOADING, true);
    match story.continue_story() {
        Ok(event) => {
            let ended = matches!(event, VmEvent::End);
            queue_event_and_following_visuals(&mut story, &mut queue, event);
            if ended {
                flags.set(PlayerFlags::ENDED, true);
                *mode = PlayerMode::Ended;
            } else {
                *mode = PlayerMode::Playing;
            }
        }
        Err(_) => *mode = PlayerMode::RuntimeError,
    }
}
