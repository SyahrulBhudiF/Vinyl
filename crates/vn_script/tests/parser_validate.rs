use std::fs;
use std::path::Path;
use vn_core::{StmtKind, compile};
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
fn parses_dialogue_and_menu_text_ids() {
    let script = parse_source(
        "locale.vn",
        r#"label start:
    eileen [intro.hello] "Hello."
    [intro.narration] "Narration."
    menu:
        [intro.ask] "Ask":
            end
"#,
    )
    .unwrap();

    let StmtKind::Say { text_id, text, .. } = &script.statements[1].kind else {
        panic!("expected speaker line");
    };
    assert_eq!(text_id.as_deref(), Some("intro.hello"));
    assert_eq!(text, "Hello.");

    let StmtKind::Say { text_id, text, .. } = &script.statements[2].kind else {
        panic!("expected narration line");
    };
    assert_eq!(text_id.as_deref(), Some("intro.narration"));
    assert_eq!(text, "Narration.");

    let StmtKind::Menu { choices } = &script.statements[3].kind else {
        panic!("expected menu");
    };
    assert_eq!(choices[0].text_id.as_deref(), Some("intro.ask"));
    assert_eq!(choices[0].text, "Ask");

    let program = compile(&script);
    let vn_core::OpKind::Say { text_id, .. } = &program.ops[0].kind else {
        panic!("expected compiled say");
    };
    assert_eq!(text_id.as_deref(), Some("intro.hello"));
}

#[test]
fn requires_exactly_one_start_label() {
    let missing = parse_source("missing.vn", "label other:\n    end\n").unwrap();
    let error = validate(&missing, Path::new(".")).unwrap_err();
    assert!(error.diagnostics().iter().any(|diagnostic| {
        diagnostic.render().contains("missing.vn:1:1")
            && diagnostic.message == "missing required label 'start'"
    }));

    let duplicate = parse_source(
        "duplicate.vn",
        "label start:\n    end\nlabel start:\n    end\n",
    )
    .unwrap();
    let error = validate(&duplicate, Path::new(".")).unwrap_err();
    assert!(error.diagnostics().iter().any(|diagnostic| {
        diagnostic.render().contains("duplicate.vn:3:1")
            && diagnostic.message == "duplicate label 'start'"
    }));
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
