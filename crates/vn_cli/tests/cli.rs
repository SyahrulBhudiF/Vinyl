use std::fs;
use std::process::Command;

#[test]
fn check_fixture_project() {
    let output = Command::new(env!("CARGO_BIN_EXE_vn_cli"))
        .args(["check", "../../fixtures/mvp"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "ok\n");
}

#[test]
fn run_fixture_project_through_menu_save_and_rollback() {
    let output = Command::new(env!("CARGO_BIN_EXE_vn_cli"))
        .args(["run", "../../fixtures/mvp"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("scene:bg room"));
    assert!(stdout.contains("show:eileen:happy:center"));
    assert!(stdout.contains("say:eileen:Hello."));
    assert!(stdout.contains("menu:Continue|Leave"));
    assert!(stdout.contains("say:eileen:Good."));
    assert!(stdout.contains("rollback:menu:Continue|Leave"));
}

#[test]
fn extract_locales_prints_fluent_entries() {
    let root = temp_project("extract");
    fs::create_dir_all(root.join("script")).unwrap();
    fs::write(
        root.join("script/start.vn"),
        r#"label start:
    eileen [intro-hello] "Hello."
    menu:
        [intro-ask] "Ask":
            end
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vn_cli"))
        .arg("extract-locales")
        .arg(&root)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("intro-ask = Ask"));
    assert!(stdout.contains("intro-hello = Hello."));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn run_uses_requested_locale() {
    let root = temp_project("locale_run");
    fs::create_dir_all(root.join("script")).unwrap();
    fs::create_dir_all(root.join("locale")).unwrap();
    fs::write(
        root.join("script/start.vn"),
        r#"label start:
    eileen [intro-hello] "Hello."
    menu:
        [intro-continue] "Continue":
            eileen [intro-good] "Good."
"#,
    )
    .unwrap();
    fs::write(
        root.join("locale/id-ID.ftl"),
        "intro-hello = Halo.\nintro-continue = Lanjut\nintro-good = Bagus.\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vn_cli"))
        .args(["run", "--locale", "id-ID"])
        .arg(&root)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("say:eileen:Halo."));
    assert!(stdout.contains("menu:Lanjut"));
    assert!(stdout.contains("say:eileen:Bagus."));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn check_reports_parse_errors_from_multiple_files() {
    let root = temp_project("parse_many");
    fs::create_dir_all(root.join("script")).unwrap();
    fs::write(root.join("script/a.vn"), "wat\nnope\n").unwrap();
    fs::write(root.join("script/b.vn"), "bad\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vn_cli"))
        .arg("check")
        .arg(&root)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("a.vn:1:1: unknown statement"));
    assert!(stderr.contains("a.vn:2:1: unknown statement"));
    assert!(stderr.contains("b.vn:1:1: unknown statement"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn check_reports_all_missing_labels_and_assets() {
    let output = Command::new(env!("CARGO_BIN_EXE_vn_cli"))
        .args(["check", "../../fixtures/bad_missing"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("start.vn:2:5"));
    assert!(stderr.contains("missing asset"));
    assert!(stderr.contains("start.vn:3:5"));
    assert!(stderr.contains("missing label 'nowhere'"));
}

fn temp_project(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("vn_cli_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}
