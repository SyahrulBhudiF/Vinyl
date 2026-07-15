use super::*;

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct VisualTestInput<'w, 's> {
    test: Option<ResMut<'w, VisualTestState>>,
    mode: Res<'w, PlayerMode>,
    loading: Res<'w, AssetLoadingState>,
    transitions: Query<'w, 's, (), With<TransitionAlpha>>,
    reveals: Query<'w, 's, (), With<TextReveal>>,
    menu: Query<'w, 's, (), With<PresentationMenu>>,
    dialogue: Query<'w, 's, &'static PresentationDialogue>,
    music: Query<'w, 's, (), With<crate::MusicRender>>,
    save_context: Res<'w, PlayerSaveContext>,
    story: ResMut<'w, VnStory>,
    queue: ResMut<'w, PresentationCommandQueue>,
    exit: MessageWriter<'w, AppExit>,
}
pub(super) fn visual_test_driver(mut commands: Commands, input: VisualTestInput) {
    let VisualTestInput {
        mut test,
        mode,
        loading,
        transitions,
        reveals,
        menu,
        dialogue,
        music,
        save_context,
        mut story,
        mut queue,
        mut exit,
    } = input;
    let Some(test) = test.as_deref_mut() else {
        return;
    };
    if *mode == PlayerMode::RuntimeError {
        exit.write(AppExit::error());
        return;
    }
    let stable = *mode == PlayerMode::Playing
        && loading.started_at.is_none()
        && loading.error.is_none()
        && transitions.is_empty()
        && reveals.is_empty();
    match test.phase {
        VisualTestPhase::WaitDialogue => {
            if stable
                && menu.is_empty()
                && !dialogue.is_empty()
                && !music.is_empty()
                && let Ok(event) = story.continue_story()
            {
                let _ = queue_event_and_following_visuals(&mut story, &mut queue, event);
                test.phase = VisualTestPhase::WaitMenu;
            }
        }
        VisualTestPhase::WaitMenu => {
            if stable && !menu.is_empty() && !dialogue.is_empty() && !music.is_empty() {
                test.stable_frames = test.stable_frames.saturating_add(1);
                if test.stable_frames >= 5 {
                    let output = test.output.join("menu.png");
                    commands
                        .spawn((Screenshot::primary_window(), VisualTestCapture))
                        .observe(move |capture: On<ScreenshotCaptured>| {
                            write_capture(&capture.image, &output);
                        });
                    test.phase = VisualTestPhase::CaptureMenu;
                }
            } else {
                test.stable_frames = 0;
            }
        }
        VisualTestPhase::CaptureMenu => {
            if !test.output.join("menu.png").exists() {
                return;
            }
            commands.insert_resource(PendingChoice(0));
            test.phase = VisualTestPhase::WaitNextDialogue;
            test.stable_frames = 0;
        }
        VisualTestPhase::WaitNextDialogue => {
            let next_dialogue = dialogue.iter().any(|dialogue| dialogue.text == "Good.");
            if stable && next_dialogue && menu.is_empty() {
                test.stable_frames = test.stable_frames.saturating_add(1);
                if test.stable_frames >= 5 {
                    let output = test.output.join("next.png");
                    commands
                        .spawn((Screenshot::primary_window(), VisualTestCapture))
                        .observe(move |capture: On<ScreenshotCaptured>| {
                            write_capture(&capture.image, &output);
                        });
                    test.phase = VisualTestPhase::CaptureNextDialogue;
                }
            } else {
                test.stable_frames = 0;
            }
        }
        VisualTestPhase::CaptureNextDialogue => {
            if !test.output.join("next.png").exists() {
                return;
            }
            capture_save(
                &mut commands,
                save_file(&story, &save_context),
                SaveSlot::Manual(1),
            );
            test.phase = VisualTestPhase::WaitSave;
        }
        VisualTestPhase::WaitSave => {
            if matches!(
                inspect_save(
                    &save_context.directory,
                    SaveSlot::Manual(1),
                    &save_context.project_id,
                    &save_context.project_version,
                    &save_context.script_hash,
                ),
                Ok(SaveSlotState::Compatible(save)) if !save.screenshot_png.is_empty()
            ) {
                exit.write(AppExit::Success);
            }
        }
    }
}

pub(super) fn write_capture(image: &Image, output: &std::path::Path) {
    let result = image
        .clone()
        .try_into_dynamic()
        .map_err(|error| error.to_string())
        .and_then(|image| image.save(output).map_err(|error| error.to_string()));
    if let Err(error) = result {
        eprintln!("visual capture failed for {}: {error}", output.display());
    }
}
