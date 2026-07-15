use super::*;
pub(super) fn sync_save_load_ui(
    mut commands: Commands,
    mode: Res<PlayerMode>,
    screen: Res<SaveLoadScreen>,
    save_context: Res<PlayerSaveContext>,
    font: Res<PlayerFont>,
    mut images: ResMut<Assets<Image>>,
    existing: Query<Entity, With<SaveLoadOverlay>>,
) {
    if !screen.is_changed() && !mode.is_changed() {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    if !matches!(*mode, PlayerMode::Save | PlayerMode::Load) {
        return;
    }
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
                Text::new(if *mode == PlayerMode::Save {
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
                        let writable = *mode == PlayerMode::Save && slot != SaveSlot::Autosave;
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

pub(super) fn slot_label(slot: SaveSlot, state: &SaveSlotState, confirm: Option<u8>) -> String {
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

pub(super) fn format_timestamp(timestamp: i64) -> String {
    chrono::DateTime::from_timestamp(timestamp, 0)
        .map(|timestamp| {
            timestamp
                .with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M")
                .to_string()
        })
        .unwrap_or_else(|| "Unknown time".to_string())
}

pub(super) fn slot_thumbnail(
    state: &SaveSlotState,
    images: &mut Assets<Image>,
) -> Option<Handle<Image>> {
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

pub(super) fn save_load_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    loading: Res<AssetLoadingState>,
    transitions: Query<(), With<TransitionAlpha>>,
    mut screen: ResMut<SaveLoadScreen>,
    mut mode: ResMut<PlayerMode>,
) {
    if loading.started_at.is_some() || loading.error.is_some() || !transitions.is_empty() {
        return;
    }
    if keys.just_pressed(KeyCode::F5) {
        screen.confirm_overwrite = None;
        *mode = PlayerMode::Save;
    } else if keys.just_pressed(KeyCode::F9) {
        screen.confirm_overwrite = None;
        *mode = PlayerMode::Load;
    } else if keys.just_pressed(KeyCode::Escape)
        && matches!(*mode, PlayerMode::Save | PlayerMode::Load)
    {
        screen.confirm_overwrite = None;
        *mode = PlayerMode::Playing;
    }
}

pub(super) type SaveLoadButtonItem<'a> = (
    Entity,
    &'a Interaction,
    Option<&'a SaveSlotButton>,
    Option<&'a SaveLoadBack>,
    Option<&'a RollbackButton>,
);

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct SaveLoadButtonInput<'w, 's> {
    buttons: Query<'w, 's, SaveLoadButtonItem<'static>, Changed<Interaction>>,
    story: ResMut<'w, VnStory>,
    presentation: ResMut<'w, crate::VnPresentation>,
    queue: ResMut<'w, PresentationCommandQueue>,
    screen: ResMut<'w, SaveLoadScreen>,
    mode: ResMut<'w, PlayerMode>,
    save_context: Res<'w, PlayerSaveContext>,
    music: Query<'w, 's, Entity, With<crate::components::PresentationMusic>>,
}

pub(super) fn save_load_button_input(mut commands: Commands, input: SaveLoadButtonInput) {
    let SaveLoadButtonInput {
        buttons,
        mut story,
        mut presentation,
        mut queue,
        mut screen,
        mut mode,
        save_context,
        music,
    } = input;
    for (_, interaction, slot, back, rollback) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if back.is_some() {
            screen.confirm_overwrite = None;
            *mode = PlayerMode::Playing;
            continue;
        }
        if rollback.is_some() {
            commands.insert_resource(PendingRollback);
            screen.confirm_overwrite = None;
            *mode = PlayerMode::Playing;
            continue;
        }
        let Some(slot) = slot else {
            continue;
        };
        match *mode {
            PlayerMode::Load => {
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
                *mode = PlayerMode::Playing;
            }
            PlayerMode::Save => {
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
                screen.confirm_overwrite = None;
                *mode = PlayerMode::Playing;
            }
            _ => {}
        }
    }
}

pub(super) fn capture_save(commands: &mut Commands, save: SaveFile, slot: SaveSlot) {
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

pub(super) fn restore_save(
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

pub(super) fn save_file(story: &VnStory, save_context: &PlayerSaveContext) -> SaveFile {
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

pub(super) fn autosave_story(
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
    let revision = story.revision();
    if save_context.autosaved_revision == Some(revision) {
        return;
    }
    let save = save_file(&story, &save_context);
    match write_save(&save_context.directory, SaveSlot::Autosave, &save) {
        Ok(_) => {
            save_context.autosaved_revision = Some(revision);
            save_context.quit_confirmed_revision = None;
        }
        Err(error) => eprintln!("autosave failed: {error}"),
    }
}
