use bevy::prelude::*;

/// Marker entity for the active background.
#[derive(Clone, Debug, Eq, PartialEq, Component)]
pub struct PresentationBackground {
    pub image: String,
}

/// Marker entity for a visible sprite.
#[derive(Clone, Debug, Eq, PartialEq, Component)]
pub struct PresentationSprite {
    pub tag: String,
    pub attrs: Vec<String>,
    pub position: String,
}

/// Marker entity for the active dialogue line.
#[derive(Clone, Debug, Eq, PartialEq, Component)]
pub struct PresentationDialogue {
    pub speaker: Option<String>,
    pub text: String,
}

/// Marker entity for the active menu.
#[derive(Clone, Debug, Eq, PartialEq, Component)]
pub struct PresentationMenu {
    pub choices: Vec<String>,
}

/// Marker entity for the active music track.
#[derive(Clone, Debug, Eq, PartialEq, Component)]
pub struct PresentationMusic {
    pub path: String,
}
