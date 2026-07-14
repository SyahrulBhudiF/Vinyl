use std::fs;
use vn_core::{Vm, VmEvent, compile};
use vn_script::load_project;

#[test]
fn loads_manifest_metadata() {
    let loaded = load_project("../../fixtures/mvp").unwrap();
    assert_eq!(loaded.manifest.project.id, "mvp");
    assert_eq!(loaded.manifest.paths.script.to_string_lossy(), "script");
    assert!(!loaded.script_hash.is_empty());
}

#[test]
fn missing_manifest_uses_root_name_defaults() {
    let dir = temp_project("defaults");
    fs::create_dir_all(dir.join("script")).unwrap();
    fs::write(dir.join("script/start.vn"), "label start:\n    end\n").unwrap();

    let loaded = load_project(&dir).unwrap();
    assert_eq!(
        loaded.manifest.project.id,
        dir.file_name().unwrap().to_string_lossy()
    );
    assert_eq!(loaded.manifest.paths.script.to_string_lossy(), "script");

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn multi_file_project_starts_at_start_label_not_first_file() {
    let dir = temp_project("start_order");
    fs::create_dir_all(dir.join("script")).unwrap();
    fs::write(dir.join("script/a.vn"), "\"wrong\"\n").unwrap();
    fs::write(dir.join("script/z.vn"), "label start:\n    \"right\"\n").unwrap();

    let loaded = load_project(&dir).unwrap();
    let mut vm = Vm::new(compile(&loaded.script)).unwrap();
    assert!(matches!(
        vm.continue_until_interaction(),
        Ok(VmEvent::Dialogue { ref text, .. }) if text == "right"
    ));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn hash_changes_when_script_content_changes() {
    let dir = temp_project("content_hash");
    fs::create_dir_all(dir.join("script")).unwrap();
    let file = dir.join("script/start.vn");
    fs::write(&file, "label start:\n    end\n").unwrap();
    let first = load_project(&dir).unwrap().script_hash;

    fs::write(&file, "label start:\n    \"changed\"\n    end\n").unwrap();
    let second = load_project(&dir).unwrap().script_hash;

    assert_ne!(first, second);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn hash_changes_when_script_path_changes() {
    let dir = temp_project("path_hash");
    fs::create_dir_all(dir.join("script/a")).unwrap();
    fs::create_dir_all(dir.join("script/b")).unwrap();
    fs::write(dir.join("script/a/start.vn"), "label start:\n    end\n").unwrap();
    let first = load_project(&dir).unwrap().script_hash;

    fs::rename(dir.join("script/a/start.vn"), dir.join("script/b/start.vn")).unwrap();
    let second = load_project(&dir).unwrap().script_hash;

    assert_ne!(first, second);
    let _ = fs::remove_dir_all(dir);
}

fn temp_project(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("vn_script_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}
