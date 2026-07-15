use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bevy::asset::{AssetPlugin, RenderAssetUsages};
#[cfg(feature = "audio")]
use bevy::audio::{AudioSink, AudioSinkPlayback, Volume};
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::window::{PrimaryWindow, WindowMode, WindowResolution};
use bevy::winit::{UpdateMode, WinitSettings};
use vn_core::{CURRENT_SAVE_VERSION, Preferences, Program, ProjectId, SaveFile, VmError, VmEvent};
use vn_runtime::{
    SaveSlot, SaveSlotState, inspect_save, project_save_dir, read_preferences, write_preferences,
    write_save,
};
use vn_script::ProjectManifest;

use crate::{
    AssetLoadingState, MenuClickGuard, MenuFocus, PendingChoice, PendingRollback,
    PresentationCommandQueue, VnAssetResolver, VnBevyPlugin, VnRenderable, VnStory,
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
    pub visual_test: Option<VisualTestConfig>,
}

/// Deterministic rendered-player capture used by Linux visual CI.
pub struct VisualTestConfig {
    pub output: PathBuf,
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

/// Validates VM construction, then runs the reusable desktop Bevy player.
pub fn run_player(config: PlayerConfig) -> Result<(), VmError> {
    let story = VnStory::with_translations(config.program.clone(), config.translations.clone())?;
    let save_directory = project_save_dir(&config.project_id.0).unwrap_or_else(|error| {
        eprintln!("save directory unavailable: {error}");
        config.project_root.join(".vinyl/saves")
    });
    let preferences = read_preferences(&save_directory)
        .map(sanitize_preferences)
        .unwrap_or_else(|error| {
            eprintln!("preferences unavailable: {error}");
            Preferences::default()
        });
    let fullscreen = preferences.fullscreen;
    let save_context = PlayerSaveContext {
        program: config.program,
        translations: config.translations,
        directory: save_directory.clone(),
        engine_version: config.engine_version,
        project_id: config.project_id.clone(),
        project_version: config.project_version,
        script_hash: config.script_hash,
        autosaved_revision: None,
        quit_confirmed_revision: None,
    };
    let visual_test = config.visual_test.map(|config| {
        if let Err(error) = std::fs::create_dir_all(&config.output) {
            eprintln!("visual output unavailable: {error}");
        }
        VisualTestState::new(config)
    });
    let asset_root = config
        .project_root
        .canonicalize()
        .unwrap_or_else(|_| config.project_root.clone())
        .to_string_lossy()
        .into_owned();
    let mut app = App::new();
    if let Some(visual_test) = visual_test {
        app.insert_resource(visual_test);
    }
    app.insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.06)))
        .insert_resource(PlayerMode::Boot)
        .insert_resource(PlayerPreferences {
            value: preferences,
            directory: save_directory,
        })
        .insert_resource(SaveLoadScreen::default())
        .insert_resource(AutoAdvance::default())
        .insert_resource(save_context)
        .insert_resource(story)
        .insert_resource(VnAssetResolver::with_manifest(
            config.project_root,
            config.manifest,
        ))
        .insert_resource(VnRenderable(true))
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::Continuous,
            unfocused_mode: UpdateMode::reactive_low_power(Duration::from_secs_f64(1.0 / 30.0)),
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
                        mode: if fullscreen {
                            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
                        } else {
                            WindowMode::Windowed
                        },
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
                pause_shortcuts,
                pause_button_input,
                settings_button_input,
                save_load_shortcuts,
                save_load_button_input,
                sync_player_state,
                autosave_story,
                sync_story_ui,
                sync_menu_focus,
                sync_pause_ui,
                sync_settings_ui,
                sync_save_load_ui,
                update_dialogue_text,
                menu_button_input,
                apply_text_speed,
                auto_advance_story,
                apply_audio_preferences,
                runtime_error_quit,
                visual_test_driver,
            )
                .chain(),
        )
        .run();
    Ok(())
}

#[derive(Resource)]
struct PlayerPreferences {
    value: Preferences,
    directory: PathBuf,
}

impl PlayerPreferences {
    fn persist(&self) {
        if let Err(error) = write_preferences(&self.directory, &self.value) {
            eprintln!("preferences save failed: {error}");
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VisualTestPhase {
    WaitDialogue,
    WaitMenu,
    CaptureMenu,
    WaitNextDialogue,
    CaptureNextDialogue,
    WaitSave,
}

#[derive(Resource)]
struct VisualTestState {
    output: PathBuf,
    phase: VisualTestPhase,
    stable_frames: u8,
}

impl VisualTestState {
    fn new(config: VisualTestConfig) -> Self {
        Self {
            output: config.output,
            phase: VisualTestPhase::WaitDialogue,
            stable_frames: 0,
        }
    }
}

#[derive(Component)]
struct VisualTestCapture;

#[derive(Resource, Default)]
struct AutoAdvance {
    dialogue_pc: Option<usize>,
    elapsed_ms: u32,
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
    autosaved_revision: Option<u64>,
    quit_confirmed_revision: Option<u64>,
}

#[derive(Component)]
struct PlayerOverlay;

#[derive(Component)]
struct RuntimeErrorQuit;

#[derive(Component)]
struct PauseOverlay;

#[derive(Component)]
struct PauseAction(PlayerAction);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PlayerAction {
    Resume,
    Save,
    Load,
    Settings,
    Rollback,
    Quit,
}

#[derive(Component)]
struct SettingsOverlay;

#[derive(Component)]
struct SettingsAction(SettingsActionKind);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SettingsActionKind {
    TextSpeed,
    AutoAdvance,
    VolumeDown,
    VolumeUp,
    Mute,
    Fullscreen,
    Back,
}

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
    confirm_overwrite: Option<u8>,
}

#[derive(Resource)]
struct PlayerFont(Handle<Font>);

fn install_bundled_font(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    let font = Font::try_from_bytes(include_bytes!("../assets/fonts/NotoSans.ttf").to_vec())
        .expect("bundled Noto Sans font is valid");
    commands.insert_resource(PlayerFont(fonts.add(font)));
}

mod menu;
mod save;
mod story;
mod visual_test;

use menu::*;
use save::*;
use story::*;
use visual_test::*;
