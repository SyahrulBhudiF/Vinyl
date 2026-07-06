use std::path::PathBuf;

use vn_core::{PresentationSnapshot, SpriteSnapshot};
use vn_runtime::AssetResolver;
use vn_script::ProjectManifest;

fn path(parts: &[&str]) -> PathBuf {
    parts.iter().collect()
}

#[test]
fn resolver_maps_script_asset_ids_to_project_relative_paths() {
    let resolver = AssetResolver::new("game");

    assert_eq!(
        resolver.background("bg school hallway"),
        path(&["game", "assets", "bg", "school", "hallway.png"])
    );
    assert_eq!(
        resolver.background("title screen"),
        path(&["game", "assets", "bg", "title", "screen.png"])
    );
    assert_eq!(
        resolver.sprite("eileen", &["happy".to_string(), "casual".to_string()]),
        path(&["game", "assets", "sprites", "eileen", "happy_casual.png"])
    );
    assert_eq!(
        resolver.sprite("eileen", &[]),
        path(&["game", "assets", "sprites", "eileen", "default.png"])
    );
    assert_eq!(
        resolver.audio("assets/audio/bgm/theme.ogg"),
        path(&["game", "assets", "audio", "bgm", "theme.ogg"])
    );
}

#[test]
fn manifest_custom_path_resolution() {
    let manifest = toml::from_str::<ProjectManifest>(
        r#"
[paths]
script = "story"
assets = "media"
locales = "l10n"

[assets]
backgrounds = "backs"
sprites = "chars"
audio = "sound"
"#,
    )
    .unwrap();
    let resolver = AssetResolver::with_manifest("game", manifest);

    assert_eq!(
        resolver.background("bg room"),
        path(&["game", "media", "backs", "room.png"])
    );
    assert_eq!(
        resolver.sprite("eileen", &["happy".to_string()]),
        path(&["game", "media", "chars", "eileen", "happy.png"])
    );
    assert_eq!(
        resolver.audio("bgm/theme.ogg"),
        path(&["game", "media", "sound", "bgm", "theme.ogg"])
    );
}

#[test]
fn snapshot_resolution_returns_only_visible_assets_in_stable_sprite_order() {
    let resolver = AssetResolver::new("game");
    let mut snapshot = PresentationSnapshot {
        background: Some("bg club room".to_string()),
        music: Some("assets/audio/bgm/club.ogg".to_string()),
        ..Default::default()
    };
    snapshot.sprites.insert(
        "zara".to_string(),
        SpriteSnapshot {
            attrs: vec!["neutral".to_string()],
            position: "right".to_string(),
        },
    );
    snapshot.sprites.insert(
        "amy".to_string(),
        SpriteSnapshot {
            attrs: vec!["surprised".to_string(), "school".to_string()],
            position: "left".to_string(),
        },
    );

    let resolved = resolver.resolve_snapshot(&snapshot);

    assert_eq!(
        resolved.background,
        Some(path(&["game", "assets", "bg", "club", "room.png"]))
    );
    assert_eq!(
        resolved.sprites,
        vec![
            (
                "amy".to_string(),
                path(&["game", "assets", "sprites", "amy", "surprised_school.png",]),
            ),
            (
                "zara".to_string(),
                path(&["game", "assets", "sprites", "zara", "neutral.png"]),
            ),
        ]
    );
    assert_eq!(
        resolved.music,
        Some(path(&["game", "assets", "audio", "bgm", "club.ogg"]))
    );
}
