use crate::vm::VmState;
use serde::{Deserialize, Serialize};
use soa_rs::{SoaClone, Soars};
use std::collections::HashMap;
use thiserror::Error;

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

/// Serializable interaction checkpoint used by rollback and saves.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, SoaClone, Soars)]
#[soa_derive(Debug, Eq, PartialEq, include(Ref), Serialize)]
pub struct RollbackCheckpoint {
    pub vm: VmState,
    pub presentation: PresentationSnapshot,
}

/// Current supported save schema version.
pub const CURRENT_SAVE_VERSION: u32 = 2;

/// Save file containing deterministic state only.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SaveFile {
    pub save_version: u32,
    pub engine_version: String,
    pub game_id: ProjectId,
    pub project_version: String,
    pub script_hash: String,
    pub vm: VmState,
    pub presentation: PresentationSnapshot,
    pub rollback: soa_rs::Soa<RollbackCheckpoint>,
    pub screenshot_png: Vec<u8>,
    pub timestamp: i64,
}

/// Save compatibility validation error.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum SaveValidationError {
    #[error("unsupported save version {actual}, expected {expected}")]
    UnsupportedSaveVersion { actual: u32, expected: u32 },
    #[error("wrong game id '{actual}', expected '{expected}'")]
    WrongGameId { actual: String, expected: String },
    #[error("wrong project version '{actual}', expected '{expected}'")]
    WrongProjectVersion { actual: String, expected: String },
    #[error("script hash mismatch '{actual}', expected '{expected}'")]
    ScriptHashMismatch { actual: String, expected: String },
}

pub fn validate_save(
    save: &SaveFile,
    project_id: &ProjectId,
    project_version: &str,
    script_hash: &str,
) -> Result<(), SaveValidationError> {
    if save.save_version != CURRENT_SAVE_VERSION {
        return Err(SaveValidationError::UnsupportedSaveVersion {
            actual: save.save_version,
            expected: CURRENT_SAVE_VERSION,
        });
    }
    if &save.game_id != project_id {
        return Err(SaveValidationError::WrongGameId {
            actual: save.game_id.0.clone(),
            expected: project_id.0.clone(),
        });
    }
    if save.project_version != project_version {
        return Err(SaveValidationError::WrongProjectVersion {
            actual: save.project_version.clone(),
            expected: project_version.to_string(),
        });
    }
    if save.script_hash != script_hash {
        return Err(SaveValidationError::ScriptHashMismatch {
            actual: save.script_hash.clone(),
            expected: script_hash.to_string(),
        });
    }
    Ok(())
}

/// User preferences persisted beside saves.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Preferences {
    pub text_speed: u16,
    pub auto_advance: bool,
    pub locale: Option<String>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            text_speed: 30,
            auto_advance: false,
            locale: None,
        }
    }
}
