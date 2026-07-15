use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bevy::asset::{AssetPlugin, RenderAssetUsages};
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::window::WindowResolution;
use bevy::winit::{UpdateMode, WinitSettings};
use vn_core::{CURRENT_SAVE_VERSION, Program, ProjectId, SaveFile, VmError, VmEvent};
use vn_runtime::{SaveSlot, SaveSlotState, inspect_save, project_save_dir, write_save};
use vn_script::ProjectManifest;

use crate::{
    AssetLoadingState, MenuFocus, PendingChoice, PendingRollback, PresentationCommandQueue,
    VnAssetResolver, VnBevyPlugin, VnRenderable, VnStory,
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
    pub project_version: String,
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
    let story = VnStory::with_translations(config.program.clone(), config.translations.clone())?;
    let save_context = PlayerSaveContext {
        program: config.program,
        translations: config.translations,
        directory: project_save_dir(&config.project_id.0).unwrap_or_else(|error| {
            eprintln!("save directory unavailable: {error}");
            config.project_root.join(".vinyl/saves")
        }),
        engine_version: config.engine_version,
        project_id: config.project_id.clone(),
        project_version: config.project_version,
        script_hash: config.script_hash,
        last_saved_pc: None,
    };
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
        .insert_resource(SaveLoadScreen::default())
        .insert_resource(save_context)
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
                save_load_shortcuts,
                save_load_button_input,
                sync_player_state,
                autosave_story,
                sync_story_ui,
                sync_save_load_ui,
                update_dialogue_text,
                menu_button_input,
                runtime_error_quit,
            )
                .chain(),
        )
        .run();
    Ok(())
}

#[derive(Resource)]
struct PlayerSaveContext {
    program: Program,
    translations: HashMap<String, String>,
    directory: PathBuf,
    engine_version: String,
    project_id: ProjectId,
    project_version: String,
    script_hash: String,
    last_saved_pc: Option<usize>,
}

#[derive(Component)]
struct PlayerOverlay;

#[derive(Component)]
struct RuntimeErrorQuit;

#[derive(Component)]
struct SaveLoadOverlay;

#[derive(Component)]
struct SaveLoadBack;

#[derive(Component)]
struct SaveSlotButton(SaveSlot);

#[derive(Component)]
struct RollbackButton;

#[derive(Component)]
struct StoryUi {
    dialogue: String,
    choices: Vec<String>,
}

#[derive(Component)]
struct DialogueText;

#[derive(Component)]
struct ChoiceButton(usize);

#[derive(Component)]
struct ManualSaveCapture;

#[derive(Resource, Default)]
struct SaveLoadScreen {
    mode: Option<PlayerMode>,
    confirm_overwrite: Option<u8>,
}

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

fn sync_save_load_ui(
    mut commands: Commands,
    screen: Res<SaveLoadScreen>,
    save_context: Res<PlayerSaveContext>,
    font: Res<PlayerFont>,
    mut images: ResMut<Assets<Image>>,
    existing: Query<Entity, With<SaveLoadOverlay>>,
) {
    if !screen.is_changed() {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    let Some(mode) = screen.mode else {
        return;
    };
    let font = font.0.clone();
    commands
        .spawn((
            SaveLoadOverlay,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                padding: UiRect::all(px(30)),
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                ..default()
            },
            BackgroundColor(Color::srgba(0.015, 0.02, 0.035, 0.97)),
            GlobalZIndex(90),
        ))
        .with_children(|overlay| {
            overlay.spawn((
                Text::new(if mode == PlayerMode::Save {
                    "Save"
                } else {
                    "Load"
                }),
                TextFont {
                    font: font.clone(),
                    font_size: 36.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            overlay
                .spawn(Node {
                    width: percent(100),
                    flex_grow: 1.0,
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::flex(2, 1.0),
                    grid_auto_rows: GridTrack::px(112.0),
                    column_gap: px(12),
                    row_gap: px(12),
                    ..default()
                })
                .with_children(|grid| {
                    for slot in
                        std::iter::once(SaveSlot::Autosave).chain((1..=12).map(SaveSlot::Manual))
                    {
                        let state = inspect_save(
                            &save_context.directory,
                            slot,
                            &save_context.project_id,
                            &save_context.project_version,
                            &save_context.script_hash,
                        )
                        .unwrap_or_else(|error| SaveSlotState::Corrupt(error.to_string()));
                        let loadable = matches!(state, SaveSlotState::Compatible(_));
                        let writable = mode == PlayerMode::Save && slot != SaveSlot::Autosave;
                        let enabled = writable || loadable;
                        let label = slot_label(slot, &state, screen.confirm_overwrite);
                        let thumbnail = slot_thumbnail(&state, &mut images);
                        grid.spawn((
                            SaveSlotButton(slot),
                            Button,
                            Node {
                                width: percent(100),
                                height: percent(100),
                                padding: UiRect::all(px(8)),
                                column_gap: px(12),
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(if enabled {
                                Color::srgb(0.08, 0.11, 0.17)
                            } else {
                                Color::srgb(0.045, 0.05, 0.07)
                            }),
                            BorderRadius::all(px(6)),
                        ))
                        .with_children(|button| {
                            if let Some(image) = thumbnail {
                                button.spawn((
                                    ImageNode::new(image),
                                    Node {
                                        width: px(160),
                                        height: px(90),
                                        ..default()
                                    },
                                ));
                            }
                            button.spawn((
                                Text::new(label),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 17.0,
                                    ..default()
                                },
                                TextColor(if enabled {
                                    Color::srgb(0.92, 0.94, 1.0)
                                } else {
                                    Color::srgb(0.5, 0.52, 0.58)
                                }),
                                Node {
                                    flex_grow: 1.0,
                                    ..default()
                                },
                            ));
                        });
                    }
                });
            overlay
                .spawn(Node {
                    width: percent(100),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|actions| {
                    actions
                        .spawn((
                            RollbackButton,
                            Button,
                            Node {
                                padding: UiRect::axes(px(20), px(10)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.12, 0.2, 0.3)),
                            BorderRadius::all(px(6)),
                        ))
                        .with_child((
                            Text::new("Rollback"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    actions
                        .spawn((
                            SaveLoadBack,
                            Button,
                            Node {
                                padding: UiRect::axes(px(20), px(10)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.18, 0.18, 0.22)),
                            BorderRadius::all(px(6)),
                        ))
                        .with_child((
                            Text::new("Back"),
                            TextFont {
                                font,
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                });
        });
}

fn slot_label(slot: SaveSlot, state: &SaveSlotState, confirm: Option<u8>) -> String {
    let name = match slot {
        SaveSlot::Autosave => "Autosave".to_string(),
        SaveSlot::Manual(number) => format!("Slot {number:02}"),
    };
    match state {
        SaveSlotState::Empty => format!("{name}\nEmpty"),
        SaveSlotState::Compatible(save) | SaveSlotState::Incompatible(save, _) => {
            let status = if matches!(state, SaveSlotState::Compatible(_)) {
                "Compatible"
            } else {
                "Incompatible"
            };
            let dialogue = save.presentation.dialogue.as_ref();
            let speaker = dialogue
                .and_then(|dialogue| dialogue.speaker.as_deref())
                .unwrap_or("Narrator");
            let excerpt = dialogue.map_or("No dialogue", |dialogue| dialogue.text.as_str());
            let excerpt = excerpt.chars().take(64).collect::<String>();
            let confirm = match slot {
                SaveSlot::Manual(number) if Some(number) == confirm => "\nPress again to overwrite",
                _ => "",
            };
            format!(
                "{name} · {status}\n{}\n{speaker}: {excerpt}{confirm}",
                format_timestamp(save.timestamp)
            )
        }
        SaveSlotState::Corrupt(error) => format!("{name}\nCorrupt · {error}"),
    }
}

fn format_timestamp(timestamp: i64) -> String {
    chrono::DateTime::from_timestamp(timestamp, 0)
        .map(|timestamp| {
            timestamp
                .with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M")
                .to_string()
        })
        .unwrap_or_else(|| "Unknown time".to_string())
}

fn slot_thumbnail(state: &SaveSlotState, images: &mut Assets<Image>) -> Option<Handle<Image>> {
    let save = match state {
        SaveSlotState::Compatible(save) | SaveSlotState::Incompatible(save, _) => save,
        SaveSlotState::Empty | SaveSlotState::Corrupt(_) => return None,
    };
    let image =
        image::load_from_memory_with_format(&save.screenshot_png, image::ImageFormat::Png).ok()?;
    Some(images.add(Image::from_dynamic(
        image,
        true,
        RenderAssetUsages::RENDER_WORLD,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use soa_rs::Soa;
    use vn_core::{PresentationSnapshot, VmState};

    fn save(timestamp: i64) -> SaveFile {
        SaveFile {
            save_version: CURRENT_SAVE_VERSION,
            engine_version: "test".to_string(),
            game_id: ProjectId::from("test"),
            project_version: "1".to_string(),
            script_hash: "hash".to_string(),
            vm: VmState::default(),
            presentation: PresentationSnapshot {
                dialogue: Some(vn_core::DialogueSnapshot {
                    speaker: Some("Eileen".to_string()),
                    text: "Welcome home".to_string(),
                }),
                ..default()
            },
            rollback: Soa::new(),
            screenshot_png: Vec::new(),
            timestamp,
        }
    }

    #[test]
    fn save_slot_label_includes_metadata_and_compatibility() {
        let label = slot_label(
            SaveSlot::Manual(3),
            &SaveSlotState::Compatible(save(0)),
            Some(3),
        );

        assert!(label.contains("Slot 03 · Compatible"));
        assert!(label.contains("1970-01-01"));
        assert!(label.contains("Eileen: Welcome home"));
        assert!(label.contains("Press again to overwrite"));
    }
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
    screen: Res<SaveLoadScreen>,
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
    } else if let Some(screen_mode) = screen.mode {
        screen_mode
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

fn save_load_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    loading: Res<AssetLoadingState>,
    transitions: Query<(), With<TransitionAlpha>>,
    mut screen: ResMut<SaveLoadScreen>,
) {
    if loading.started_at.is_some() || loading.error.is_some() || !transitions.is_empty() {
        return;
    }
    if keys.just_pressed(KeyCode::F5) {
        screen.mode = Some(PlayerMode::Save);
        screen.confirm_overwrite = None;
    } else if keys.just_pressed(KeyCode::F9) {
        screen.mode = Some(PlayerMode::Load);
        screen.confirm_overwrite = None;
    } else if keys.just_pressed(KeyCode::Escape) && screen.mode.is_some() {
        screen.mode = None;
        screen.confirm_overwrite = None;
    }
}

type SaveLoadButtonItem<'a> = (
    Entity,
    &'a Interaction,
    Option<&'a SaveSlotButton>,
    Option<&'a SaveLoadBack>,
    Option<&'a RollbackButton>,
);

#[derive(bevy::ecs::system::SystemParam)]
struct SaveLoadButtonInput<'w, 's> {
    buttons: Query<'w, 's, SaveLoadButtonItem<'static>, Changed<Interaction>>,
    story: ResMut<'w, VnStory>,
    presentation: ResMut<'w, crate::VnPresentation>,
    queue: ResMut<'w, PresentationCommandQueue>,
    screen: ResMut<'w, SaveLoadScreen>,
    save_context: Res<'w, PlayerSaveContext>,
    music: Query<'w, 's, Entity, With<crate::components::PresentationMusic>>,
}

fn save_load_button_input(mut commands: Commands, input: SaveLoadButtonInput) {
    let SaveLoadButtonInput {
        buttons,
        mut story,
        mut presentation,
        mut queue,
        mut screen,
        save_context,
        music,
    } = input;
    for (_, interaction, slot, back, rollback) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if back.is_some() {
            screen.mode = None;
            screen.confirm_overwrite = None;
            continue;
        }
        if rollback.is_some() {
            commands.insert_resource(PendingRollback);
            screen.mode = None;
            screen.confirm_overwrite = None;
            continue;
        }
        let Some(slot) = slot else {
            continue;
        };
        match screen.mode {
            Some(PlayerMode::Load) => {
                let Ok(SaveSlotState::Compatible(save)) = inspect_save(
                    &save_context.directory,
                    slot.0,
                    &save_context.project_id,
                    &save_context.project_version,
                    &save_context.script_hash,
                ) else {
                    continue;
                };
                restore_save(
                    &mut commands,
                    &mut story,
                    &mut presentation,
                    &mut queue,
                    &music,
                    &save_context,
                    save,
                );
                screen.mode = None;
            }
            Some(PlayerMode::Save) => {
                let SaveSlot::Manual(number) = slot.0 else {
                    continue;
                };
                let occupied = !matches!(
                    inspect_save(
                        &save_context.directory,
                        slot.0,
                        &save_context.project_id,
                        &save_context.project_version,
                        &save_context.script_hash,
                    ),
                    Ok(SaveSlotState::Empty)
                );
                if occupied && screen.confirm_overwrite != Some(number) {
                    screen.confirm_overwrite = Some(number);
                    continue;
                }
                capture_save(&mut commands, save_file(&story, &save_context), slot.0);
                screen.mode = None;
                screen.confirm_overwrite = None;
            }
            _ => {}
        }
    }
}

fn capture_save(commands: &mut Commands, save: SaveFile, slot: SaveSlot) {
    commands
        .spawn((Screenshot::primary_window(), ManualSaveCapture))
        .observe(
            move |capture: On<ScreenshotCaptured>, context: Res<PlayerSaveContext>| {
                let mut save = save.clone();
                match capture.image.clone().try_into_dynamic() {
                    Ok(image) => {
                        let thumbnail = image
                            .resize_to_fill(320, 180, image::imageops::FilterType::Triangle)
                            .to_rgb8();
                        let mut cursor = std::io::Cursor::new(Vec::new());
                        if thumbnail
                            .write_to(&mut cursor, image::ImageFormat::Png)
                            .is_ok()
                        {
                            save.screenshot_png = cursor.into_inner();
                        }
                    }
                    Err(error) => eprintln!("screenshot conversion failed: {error}"),
                }
                if let Err(error) = write_save(&context.directory, slot, &save) {
                    eprintln!("save failed: {error}");
                }
            },
        );
}

fn restore_save(
    commands: &mut Commands,
    story: &mut VnStory,
    presentation: &mut crate::VnPresentation,
    queue: &mut PresentationCommandQueue,
    music: &Query<Entity, With<crate::components::PresentationMusic>>,
    context: &PlayerSaveContext,
    save: SaveFile,
) {
    story.restore(
        context.program.clone(),
        save.vm,
        save.presentation.clone(),
        save.rollback,
        context.translations.clone(),
    );
    presentation.snapshot = save.presentation;
    presentation.pending_commands.clear();
    queue.commands.clear();
    for entity in music {
        commands.entity(entity).despawn();
    }
}

fn save_file(story: &VnStory, save_context: &PlayerSaveContext) -> SaveFile {
    SaveFile {
        save_version: CURRENT_SAVE_VERSION,
        engine_version: save_context.engine_version.clone(),
        game_id: save_context.project_id.clone(),
        project_version: save_context.project_version.clone(),
        script_hash: save_context.script_hash.clone(),
        vm: story.vm().state().clone(),
        presentation: story.vm().presentation().clone(),
        rollback: story.vm().rollback_history().clone(),
        screenshot_png: Vec::new(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs() as i64),
    }
}

fn autosave_story(
    story: Res<VnStory>,
    presentation: Res<crate::VnPresentation>,
    mode: Res<PlayerMode>,
    loading: Res<AssetLoadingState>,
    transitions: Query<(), With<TransitionAlpha>>,
    mut save_context: ResMut<PlayerSaveContext>,
) {
    if *mode != PlayerMode::Playing
        || loading.started_at.is_some()
        || loading.error.is_some()
        || !transitions.is_empty()
        || !presentation.pending_commands.is_empty()
    {
        return;
    }
    let Some(event) = story.last_event() else {
        return;
    };
    if !matches!(event, VmEvent::Dialogue { .. } | VmEvent::Menu { .. }) {
        return;
    }
    let pc = story.vm().state().pc;
    if save_context.last_saved_pc == Some(pc) {
        return;
    }
    let save = save_file(&story, &save_context);
    match write_save(&save_context.directory, SaveSlot::Autosave, &save) {
        Ok(_) => save_context.last_saved_pc = Some(pc),
        Err(error) => eprintln!("autosave failed: {error}"),
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
