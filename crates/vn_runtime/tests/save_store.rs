use std::{fs, path::PathBuf};

use soa_rs::Soa;
use vn_core::{CURRENT_SAVE_VERSION, Preferences, ProjectId, SaveFile, VmState};
use vn_runtime::{
    SaveSlot, SaveSlotState, inspect_save, read_preferences, read_save, write_preferences,
    write_save,
};

fn directory(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("vinyl-save-store-{}-{name}", std::process::id()))
}

fn save(script_hash: &str) -> SaveFile {
    SaveFile {
        save_version: CURRENT_SAVE_VERSION,
        engine_version: "0.1.0".to_string(),
        game_id: ProjectId::from("demo"),
        project_version: "0.1.0".to_string(),
        script_hash: script_hash.to_string(),
        vm: VmState::default(),
        presentation: Default::default(),
        rollback: Soa::new(),
        screenshot_png: Vec::new(),
        timestamp: 0,
    }
}

#[test]
fn save_slots_round_trip_and_reject_invalid_numbers() {
    let directory = directory("round-trip");
    let _ = fs::remove_dir_all(&directory);

    let path = write_save(&directory, SaveSlot::Manual(1), &save("abc")).unwrap();
    assert_eq!(path.file_name().unwrap(), "slot-01.json");
    assert_eq!(
        read_save(&directory, SaveSlot::Manual(1)).unwrap(),
        save("abc")
    );
    assert!(write_save(&directory, SaveSlot::Manual(13), &save("bad")).is_err());

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn incompatible_and_corrupt_slots_remain_visible() {
    let directory = directory("inspect");
    let _ = fs::remove_dir_all(&directory);
    write_save(&directory, SaveSlot::Manual(1), &save("old")).unwrap();
    fs::write(directory.join("slot-02.json"), b"broken").unwrap();

    assert!(matches!(
        inspect_save(
            &directory,
            SaveSlot::Manual(1),
            &ProjectId::from("demo"),
            "0.1.0",
            "new"
        )
        .unwrap(),
        SaveSlotState::Incompatible(_, _)
    ));
    assert!(matches!(
        inspect_save(
            &directory,
            SaveSlot::Manual(2),
            &ProjectId::from("demo"),
            "0.1.0",
            "new"
        )
        .unwrap(),
        SaveSlotState::Corrupt(_)
    ));
    write_save(&directory, SaveSlot::Manual(1), &save("new")).unwrap();
    assert!(matches!(
        inspect_save(
            &directory,
            SaveSlot::Manual(1),
            &ProjectId::from("demo"),
            "0.1.0",
            "new"
        )
        .unwrap(),
        SaveSlotState::Compatible(_)
    ));

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn preferences_live_beside_but_outside_save_slots() {
    let directory = directory("preferences");
    let _ = fs::remove_dir_all(&directory);
    let preferences = Preferences {
        text_speed: 60,
        auto_advance: true,
        music_volume: 40,
        muted: true,
        fullscreen: true,
        locale: None,
    };

    assert_eq!(
        read_preferences(&directory).unwrap(),
        Preferences::default()
    );
    write_preferences(&directory, &preferences).unwrap();
    assert_eq!(read_preferences(&directory).unwrap(), preferences);
    assert!(!directory.join("slot-01.json").exists());

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn stale_temp_file_does_not_destroy_previous_slot() {
    let directory = directory("atomic");
    let _ = fs::remove_dir_all(&directory);
    write_save(&directory, SaveSlot::Autosave, &save("old")).unwrap();
    fs::write(directory.join("autosave.json.tmp"), b"interrupted").unwrap();

    assert_eq!(
        read_save(&directory, SaveSlot::Autosave)
            .unwrap()
            .script_hash,
        "old"
    );

    fs::remove_dir_all(directory).unwrap();
}
