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
