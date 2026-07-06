use std::fs;
use vn_script::{ProjectManifest, parse_source, validate_with_manifest};

#[test]
fn validator_uses_manifest_asset_roots_and_collects_missing_assets() {
    let dir = std::env::temp_dir().join(format!("vn_script_assets_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("media/backs")).unwrap();
    fs::write(dir.join("media/backs/room.png"), []).unwrap();

    let manifest = toml::from_str::<ProjectManifest>(
        r#"
[paths]
script = "story"
assets = "media"
locales = "locale"

[assets]
backgrounds = "backs"
sprites = "chars"
audio = "sound"
"#,
    )
    .unwrap();
    let script = parse_source(
        "test.vn",
        r#"label start:
    scene bg room
    show eileen happy at center
    play music "theme.ogg"
    end
"#,
    )
    .unwrap();

    let error = validate_with_manifest(&script, &dir, &manifest).unwrap_err();
    let rendered = error
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.render())
        .collect::<Vec<_>>()
        .join("\n");

    assert_eq!(error.diagnostics().len(), 2);
    assert!(rendered.contains("media/chars/eileen/happy.png"));
    assert!(rendered.contains("media/sound/theme.ogg"));

    let _ = fs::remove_dir_all(dir);
}
