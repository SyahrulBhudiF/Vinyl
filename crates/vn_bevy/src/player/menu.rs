use super::*;
pub(super) fn pause_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    loading: Res<AssetLoadingState>,
    mut mode: ResMut<PlayerMode>,
    mut screen: ResMut<SaveLoadScreen>,
    mut preferences: ResMut<PlayerPreferences>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    if keys.pressed(KeyCode::AltLeft) && keys.just_pressed(KeyCode::Enter) {
        preferences.value.fullscreen = !preferences.value.fullscreen;
        window.mode = if preferences.value.fullscreen {
            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
        } else {
            WindowMode::Windowed
        };
        preferences.persist();
        return;
    }
    if !keys.just_pressed(KeyCode::Escape) || loading.error.is_some() {
        return;
    }
    match *mode {
        PlayerMode::Playing => *mode = PlayerMode::Paused,
        PlayerMode::Paused => *mode = PlayerMode::Playing,
        PlayerMode::Save | PlayerMode::Load => {
            screen.confirm_overwrite = None;
            *mode = PlayerMode::Paused;
        }
        PlayerMode::Settings => *mode = PlayerMode::Paused,
        _ => {}
    }
}

pub(super) fn sync_pause_ui(
    mut commands: Commands,
    mode: Res<PlayerMode>,
    save_context: Res<PlayerSaveContext>,
    font: Res<PlayerFont>,
    existing: Query<Entity, With<PauseOverlay>>,
) {
    if !mode.is_changed() && !save_context.is_changed() {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    if *mode != PlayerMode::Paused {
        return;
    }
    let font = font.0.clone();
    commands
        .spawn((
            PauseOverlay,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                ..default()
            },
            BackgroundColor(Color::srgba(0.015, 0.02, 0.035, 0.9)),
            GlobalZIndex(80),
        ))
        .with_children(|overlay| {
            overlay.spawn((
                Text::new("Paused"),
                TextFont {
                    font: font.clone(),
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            for (action, label) in [
                (PlayerAction::Resume, "Resume"),
                (PlayerAction::Save, "Save"),
                (PlayerAction::Load, "Load"),
                (PlayerAction::Settings, "Settings"),
                (PlayerAction::Rollback, "Rollback"),
                (
                    PlayerAction::Quit,
                    if save_context.quit_confirmed_revision.is_some() {
                        "Quit — press again to confirm"
                    } else {
                        "Quit"
                    },
                ),
            ] {
                overlay
                    .spawn((
                        PauseAction(action),
                        Button,
                        Node {
                            width: px(260),
                            padding: UiRect::axes(px(22), px(10)),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.1, 0.14, 0.21)),
                        BorderRadius::all(px(6)),
                    ))
                    .with_child((
                        Text::new(label),
                        TextFont {
                            font: font.clone(),
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
            }
        });
}

pub(super) fn pause_button_input(
    mut commands: Commands,
    buttons: Query<(&Interaction, &PauseAction), Changed<Interaction>>,
    story: Res<VnStory>,
    mut mode: ResMut<PlayerMode>,
    mut save_context: ResMut<PlayerSaveContext>,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, action) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action.0 {
            PlayerAction::Resume => {
                save_context.quit_confirmed_revision = None;
                *mode = PlayerMode::Playing;
            }
            PlayerAction::Save => *mode = PlayerMode::Save,
            PlayerAction::Load => *mode = PlayerMode::Load,
            PlayerAction::Settings => *mode = PlayerMode::Settings,
            PlayerAction::Rollback => {
                commands.insert_resource(PendingRollback);
                *mode = PlayerMode::Playing;
            }
            PlayerAction::Quit => {
                let revision = story.revision();
                if save_context.autosaved_revision == Some(revision)
                    || save_context.quit_confirmed_revision == Some(revision)
                {
                    exit.write(AppExit::Success);
                } else {
                    save_context.quit_confirmed_revision = Some(revision);
                }
            }
        }
    }
}

pub(super) fn sync_settings_ui(
    mut commands: Commands,
    mode: Res<PlayerMode>,
    preferences: Res<PlayerPreferences>,
    font: Res<PlayerFont>,
    existing: Query<Entity, With<SettingsOverlay>>,
) {
    if !mode.is_changed() && !preferences.is_changed() {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    if *mode != PlayerMode::Settings {
        return;
    }
    let font = font.0.clone();
    let labels = [
        (
            SettingsActionKind::TextSpeed,
            format!(
                "Text Speed: {}",
                text_speed_name(preferences.value.text_speed)
            ),
        ),
        (
            SettingsActionKind::AutoAdvance,
            format!(
                "Auto Advance: {}",
                if preferences.value.auto_advance {
                    "On"
                } else {
                    "Off"
                }
            ),
        ),
        (SettingsActionKind::VolumeDown, "Music Volume −".to_string()),
        (
            SettingsActionKind::VolumeUp,
            format!("Music Volume +  ({}%)", preferences.value.music_volume),
        ),
        (
            SettingsActionKind::Mute,
            format!(
                "Mute: {}",
                if preferences.value.muted { "On" } else { "Off" }
            ),
        ),
        (
            SettingsActionKind::Fullscreen,
            format!(
                "Fullscreen: {}",
                if preferences.value.fullscreen {
                    "On"
                } else {
                    "Off"
                }
            ),
        ),
        (SettingsActionKind::Back, "Back".to_string()),
    ];
    commands
        .spawn((
            SettingsOverlay,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                ..default()
            },
            BackgroundColor(Color::srgba(0.015, 0.02, 0.035, 0.97)),
            GlobalZIndex(90),
        ))
        .with_children(|overlay| {
            overlay.spawn((
                Text::new("Settings"),
                TextFont {
                    font: font.clone(),
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            for (action, label) in labels {
                overlay
                    .spawn((
                        SettingsAction(action),
                        Button,
                        Node {
                            width: px(360),
                            padding: UiRect::axes(px(20), px(10)),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.1, 0.14, 0.21)),
                        BorderRadius::all(px(6)),
                    ))
                    .with_child((
                        Text::new(label),
                        TextFont {
                            font: font.clone(),
                            font_size: 21.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
            }
        });
}

pub(super) fn settings_button_input(
    buttons: Query<(&Interaction, &SettingsAction), Changed<Interaction>>,
    mut mode: ResMut<PlayerMode>,
    mut preferences: ResMut<PlayerPreferences>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    for (interaction, action) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action.0 {
            SettingsActionKind::TextSpeed => {
                preferences.value.text_speed = match preferences.value.text_speed {
                    15 => 30,
                    30 => 60,
                    60 => u16::MAX,
                    _ => 15,
                };
            }
            SettingsActionKind::AutoAdvance => {
                preferences.value.auto_advance = !preferences.value.auto_advance;
            }
            SettingsActionKind::VolumeDown => {
                preferences.value.music_volume = preferences.value.music_volume.saturating_sub(10);
            }
            SettingsActionKind::VolumeUp => {
                preferences.value.music_volume = (preferences.value.music_volume + 10).min(100);
            }
            SettingsActionKind::Mute => preferences.value.muted = !preferences.value.muted,
            SettingsActionKind::Fullscreen => {
                preferences.value.fullscreen = !preferences.value.fullscreen;
                window.mode = if preferences.value.fullscreen {
                    WindowMode::BorderlessFullscreen(MonitorSelection::Current)
                } else {
                    WindowMode::Windowed
                };
            }
            SettingsActionKind::Back => {
                *mode = PlayerMode::Paused;
                continue;
            }
        }
        preferences.persist();
    }
}

pub(super) fn sanitize_preferences(mut preferences: Preferences) -> Preferences {
    if !matches!(preferences.text_speed, 15 | 30 | 60 | u16::MAX) {
        preferences.text_speed = Preferences::default().text_speed;
    }
    preferences.music_volume = preferences.music_volume.min(100);
    preferences
}

pub(super) fn text_speed_name(speed: u16) -> &'static str {
    match speed {
        15 => "Slow",
        30 => "Normal",
        60 => "Fast",
        _ => "Instant",
    }
}
