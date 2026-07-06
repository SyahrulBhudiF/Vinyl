use bevy::prelude::*;
use std::path::Path;
use vn_bevy::{VnAssetResolver, VnBevyPlugin, VnRenderable, VnStory};
use vn_core::compile;
use vn_script::{load_project, validate};

fn main() {
    let project = Path::new("fixtures/mvp");
    let loaded = load_project(project).expect("fixture project loads");
    validate(&loaded.script, &loaded.root).expect("fixture project validates");
    let program = compile(&loaded.script);

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(VnBevyPlugin)
        .insert_resource(VnStory::new(program))
        .insert_resource(VnAssetResolver::new(project))
        .insert_resource(VnRenderable(true))
        .run();
}
