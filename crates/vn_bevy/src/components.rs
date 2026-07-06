use bevy::prelude::*;
use vn_core::{TextEffect, Transition};

/// Marker entity for the active background.
#[derive(Clone, Debug, Eq, PartialEq, Component)]
pub struct PresentationBackground {
    pub image: String,
    pub transition: Option<Transition>,
}

/// Marker entity for a visible sprite.
#[derive(Clone, Debug, Eq, PartialEq, Component)]
pub struct PresentationSprite {
    pub tag: String,
    pub attrs: Vec<String>,
    pub position: String,
    pub transition: Option<Transition>,
}

/// Marker entity for the active dialogue line.
#[derive(Clone, Debug, Eq, PartialEq, Component)]
pub struct PresentationDialogue {
    pub speaker: Option<String>,
    pub text: String,
    pub effect: TextEffect,
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

/// Transient alpha transition state.
#[derive(Clone, Debug, Eq, PartialEq, Component)]
pub struct TransitionAlpha {
    pub elapsed_ms: u32,
    pub duration_ms: u32,
}

impl TransitionAlpha {
    pub fn alpha_permille(&self) -> u32 {
        if self.duration_ms == 0 {
            return 1000;
        }
        (self.elapsed_ms.saturating_mul(1000) / self.duration_ms).min(1000)
    }
}

/// Transient typewriter reveal state.
#[derive(Clone, Debug, Eq, PartialEq, Component)]
pub struct TextReveal {
    pub elapsed_ms: u32,
    pub chars_per_second: u16,
    pub total_chars: usize,
}

impl TextReveal {
    pub fn visible_chars(&self) -> usize {
        (usize::from(self.chars_per_second) * self.elapsed_ms as usize / 1000).min(self.total_chars)
    }
}
