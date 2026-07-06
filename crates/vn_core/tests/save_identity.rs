use vn_core::{Preferences, ProjectId, SaveFile, VmState};

#[test]
fn save_file_uses_project_id_and_script_hash() {
    let save = SaveFile {
        engine_version: "0.1.0".to_string(),
        game_id: ProjectId::from("demo"),
        script_hash: "abc123".to_string(),
        vm: VmState::default(),
        presentation: Default::default(),
        preferences: Preferences::default(),
        screenshot_png: Vec::new(),
        timestamp: 0,
    };

    let json = serde_json::to_string(&save).unwrap();
    assert!(json.contains("abc123"));
    let restored: SaveFile = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.game_id, ProjectId::from("demo"));
    assert_eq!(restored.script_hash, "abc123");
}
