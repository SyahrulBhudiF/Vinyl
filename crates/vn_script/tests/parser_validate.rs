use std::fs;
use std::path::Path;
use vn_core::StmtKind;
use vn_script::{parse_source, validate};

const MVP: &str = r#"label start:
    scene bg room
    show eileen happy at center
    eileen "Hello."
    menu:
        "Continue":
            eileen "Good."
            end
        "Leave":
            "You leave."
            end
"#;

#[test]
fn parses_mvp_script() {
    let script = parse_source("test.vn", MVP).unwrap();
    assert_eq!(script.statements.len(), 5);
    assert!(matches!(script.statements[0].kind, StmtKind::Label { .. }));
    let StmtKind::Menu { choices } = &script.statements[4].kind else {
        panic!("expected menu");
    };
    assert_eq!(choices.len(), 2);
    assert_eq!(choices[0].text, "Continue");
}

#[test]
fn reports_missing_label_with_position() {
    let script = parse_source("bad.vn", "label start:\n    jump missing\n").unwrap();
    let error = validate(&script, Path::new(".")).unwrap_err();
    let rendered = error.diagnostics()[0].render();
    assert!(rendered.contains("bad.vn:2:5"));
    assert!(rendered.contains("missing label 'missing'"));
}

#[test]
fn reports_missing_asset_with_position() {
    let script = parse_source("bad.vn", "label start:\n    scene bg missing\n").unwrap();
    let dir = std::env::temp_dir().join(format!("vn_script_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let error = validate(&script, &dir).unwrap_err();
    let rendered = error.diagnostics()[0].render();
    assert!(rendered.contains("bad.vn:2:5"));
    assert!(rendered.contains("missing asset"));
    let _ = fs::remove_dir_all(&dir);
}
