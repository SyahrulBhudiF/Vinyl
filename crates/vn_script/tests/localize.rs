use std::fs;
use vn_script::{load_project, parse_locale};

#[test]
fn parses_flat_fluent_messages() {
    let catalog = parse_locale(
        "test.ftl".as_ref(),
        "id-ID",
        r#"intro-hello = Halo.
intro-ask = Tanya kota
"#,
    )
    .unwrap();

    assert_eq!(catalog.locale, "id-ID");
    assert_eq!(catalog.get("intro-hello"), Some("Halo."));
    assert_eq!(catalog.get("intro-ask"), Some("Tanya kota"));
}

#[test]
fn project_loader_reads_locale_catalogs_from_manifest_locale_root() {
    let root = std::env::temp_dir().join(format!("vn_locale_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("script")).unwrap();
    fs::create_dir_all(root.join("locale")).unwrap();
    fs::write(
        root.join("script/start.vn"),
        r#"label start:
    eileen [intro-hello] "Hello."
"#,
    )
    .unwrap();
    fs::write(root.join("locale/id-ID.ftl"), "intro-hello = Halo.\n").unwrap();

    let loaded = load_project(&root).unwrap();

    assert_eq!(loaded.locales.len(), 1);
    assert_eq!(loaded.locales[0].locale, "id-ID");
    assert_eq!(loaded.locales[0].get("intro-hello"), Some("Halo."));

    let _ = fs::remove_dir_all(&root);
}
