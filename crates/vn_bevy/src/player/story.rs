use super::*;
pub(super) fn sync_story_ui(
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
    if existing
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

pub(super) fn sync_menu_focus(
    focus: Res<MenuFocus>,
    mut choices: Query<(&ChoiceButton, &mut BackgroundColor)>,
) {
    if !focus.is_changed() {
        return;
    }
    for (choice, mut color) in &mut choices {
        *color = if choice.0 == focus.0 {
            Color::srgb(0.16, 0.25, 0.38)
        } else {
            Color::srgb(0.08, 0.11, 0.17)
        }
        .into();
    }
}

pub(super) fn apply_text_speed(
    preferences: Res<PlayerPreferences>,
    mut reveals: Query<&mut TextReveal>,
) {
    for mut reveal in &mut reveals {
        reveal.chars_per_second = preferences.value.text_speed;
        if preferences.value.text_speed == u16::MAX {
            reveal.elapsed_ms = u32::MAX;
        }
    }
}

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct AutoAdvanceInput<'w, 's> {
    time: Res<'w, Time>,
    mode: Res<'w, PlayerMode>,
    preferences: Res<'w, PlayerPreferences>,
    loading: Res<'w, AssetLoadingState>,
    transitions: Query<'w, 's, (), With<TransitionAlpha>>,
    reveals: Query<'w, 's, (), With<TextReveal>>,
    menu: Query<'w, 's, (), With<PresentationMenu>>,
    story: ResMut<'w, VnStory>,
    queue: ResMut<'w, PresentationCommandQueue>,
    auto: ResMut<'w, AutoAdvance>,
}

pub(super) fn auto_advance_story(input: AutoAdvanceInput) {
    let AutoAdvanceInput {
        time,
        mode,
        preferences,
        loading,
        transitions,
        reveals,
        menu,
        mut story,
        mut queue,
        mut auto,
    } = input;
    if !preferences.value.auto_advance
        || *mode != PlayerMode::Playing
        || loading.started_at.is_some()
        || loading.error.is_some()
        || !transitions.is_empty()
        || !reveals.is_empty()
        || !menu.is_empty()
    {
        auto.dialogue_pc = None;
        auto.elapsed_ms = 0;
        return;
    }
    let Some(VmEvent::Dialogue { text, .. }) = story.last_event() else {
        auto.dialogue_pc = None;
        auto.elapsed_ms = 0;
        return;
    };
    let pc = story.vm().state().pc;
    if auto.dialogue_pc != Some(pc) {
        auto.dialogue_pc = Some(pc);
        auto.elapsed_ms = 0;
    }
    auto.elapsed_ms = auto
        .elapsed_ms
        .saturating_add(time.delta().as_millis().min(u128::from(u32::MAX)) as u32);
    let wait_ms = 1_500u32.saturating_add(text.chars().count() as u32 * 45);
    if auto.elapsed_ms < wait_ms {
        return;
    }
    if let Ok(event) = story.continue_story() {
        let _ = queue_event_and_following_visuals(&mut story, &mut queue, event);
    }
    auto.dialogue_pc = None;
    auto.elapsed_ms = 0;
}

#[cfg(feature = "audio")]
pub(super) fn apply_audio_preferences(
    preferences: Res<PlayerPreferences>,
    mut sinks: Query<&mut AudioSink>,
) {
    if !preferences.is_changed() {
        return;
    }
    let volume = if preferences.value.muted {
        Volume::SILENT
    } else {
        Volume::Linear(f32::from(preferences.value.music_volume) / 100.0)
    };
    for mut sink in &mut sinks {
        sink.set_volume(volume);
    }
}

#[cfg(not(feature = "audio"))]
pub(super) fn apply_audio_preferences() {}

pub(super) fn update_dialogue_text(
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

pub(super) fn menu_button_input(
    mut commands: Commands,
    mut buttons: Query<(&ChoiceButton, &Interaction, &mut BackgroundColor), Changed<Interaction>>,
    loading: Res<AssetLoadingState>,
    transitions: Query<(), With<TransitionAlpha>>,
    click_guard: Res<MenuClickGuard>,
) {
    if click_guard.0
        || loading.started_at.is_some()
        || loading.error.is_some()
        || !transitions.is_empty()
    {
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

pub(super) fn sync_player_state(
    mut commands: Commands,
    loading: Res<AssetLoadingState>,
    story: Res<VnStory>,
    font: Res<PlayerFont>,
    overlays: Query<Entity, With<PlayerOverlay>>,
    mut mode: ResMut<PlayerMode>,
) {
    let error = loading.error.as_deref();
    let show_loading = error.is_none() && loading.visible();
    *mode = if error.is_some() {
        PlayerMode::RuntimeError
    } else if loading.started_at.is_some() {
        PlayerMode::Loading
    } else if matches!(
        *mode,
        PlayerMode::Paused | PlayerMode::Save | PlayerMode::Load | PlayerMode::Settings
    ) {
        *mode
    } else if story.ended() {
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

pub(super) fn runtime_error_quit(
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

pub(super) fn start_story(
    mut story: ResMut<VnStory>,
    mut queue: ResMut<PresentationCommandQueue>,
    mut mode: ResMut<PlayerMode>,
) {
    *mode = PlayerMode::Loading;
    match story.continue_story() {
        Ok(event) => {
            let _ = queue_event_and_following_visuals(&mut story, &mut queue, event);
            *mode = if story.ended() {
                PlayerMode::Ended
            } else {
                PlayerMode::Playing
            };
        }
        Err(_) => *mode = PlayerMode::RuntimeError,
    }
}
