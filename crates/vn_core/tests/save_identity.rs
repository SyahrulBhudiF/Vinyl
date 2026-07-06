use vn_core::{
    CURRENT_SAVE_VERSION, Preferences, ProjectId, SaveFile, SaveValidationError, VmState,
    validate_save,
};

fn save() -> SaveFile {
    SaveFile {
        save_version: CURRENT_SAVE_VERSION,
        engine_version: "0.1.0".to_string(),
        game_id: ProjectId::from("demo"),
        project_version: "0.1.0".to_string(),
        script_hash: "abc123".to_string(),
        vm: VmState::default(),
        presentation: Default::default(),
        preferences: Preferences::default(),
        screenshot_png: Vec::new(),
        timestamp: 0,
    }
}

#[test]
fn save_file_uses_project_id_and_script_hash() {
    let save = save();

    let json = serde_json::to_string(&save).unwrap();
    assert!(json.contains("abc123"));
    let restored: SaveFile = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.game_id, ProjectId::from("demo"));
    assert_eq!(restored.script_hash, "abc123");
}

#[test]
fn validate_save_rejects_incompatible_identity() {
    assert_eq!(
        validate_save(&save(), &ProjectId::from("demo"), "0.1.0", "abc123"),
        Ok(())
    );

    let mut wrong_hash = save();
    wrong_hash.script_hash = "wrong".to_string();
    assert_eq!(
        validate_save(&wrong_hash, &ProjectId::from("demo"), "0.1.0", "abc123"),
        Err(SaveValidationError::ScriptHashMismatch {
            actual: "wrong".to_string(),
            expected: "abc123".to_string(),
        })
    );

    let mut wrong_game = save();
    wrong_game.game_id = ProjectId::from("other");
    assert!(matches!(
        validate_save(&wrong_game, &ProjectId::from("demo"), "0.1.0", "abc123"),
        Err(SaveValidationError::WrongGameId { .. })
    ));

    let mut old = save();
    old.save_version = 0;
    assert!(matches!(
        validate_save(&old, &ProjectId::from("demo"), "0.1.0", "abc123"),
        Err(SaveValidationError::UnsupportedSaveVersion { .. })
    ));
}
