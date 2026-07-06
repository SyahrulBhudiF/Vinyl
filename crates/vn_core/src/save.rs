use crate::vm::VmState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Serializable presentation snapshot owned outside renderers.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct PresentationSnapshot {
    pub background: Option<String>,
    pub sprites: HashMap<String, SpriteSnapshot>,
    pub music: Option<String>,
    pub dialogue: Option<DialogueSnapshot>,
    pub menu: Option<Vec<String>>,
}

/// Serializable sprite presentation state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SpriteSnapshot {
    pub attrs: Vec<String>,
    pub position: String,
}

/// Serializable dialogue state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DialogueSnapshot {
    pub speaker: Option<String>,
    pub text: String,
}

/// Stable project identifier stored in saves.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub String);

impl From<String> for ProjectId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for ProjectId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// Save file containing deterministic state only.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SaveFile {
    pub engine_version: String,
    pub game_id: ProjectId,
    pub script_hash: String,
    pub vm: VmState,
    pub presentation: PresentationSnapshot,
    pub preferences: Preferences,
    pub screenshot_png: Vec<u8>,
    pub timestamp: i64,
}

/// User preferences persisted beside saves.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Preferences {
    pub text_speed: u16,
    pub auto_advance: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            text_speed: 30,
            auto_advance: false,
        }
    }
}
