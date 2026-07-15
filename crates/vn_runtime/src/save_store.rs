use std::{
    env,
    fs::{self, File},
    io::{self, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use vn_core::{ProjectId, SaveFile, SaveValidationError, validate_save};

/// A manual save slot or the single autosave slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveSlot {
    Autosave,
    Manual(u8),
}

impl SaveSlot {
    fn file_name(self) -> io::Result<String> {
        match self {
            Self::Autosave => Ok("autosave.json".to_string()),
            Self::Manual(1..=12) => Ok(format!("slot-{:02}.json", self.number())),
            Self::Manual(slot) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("save slot {slot} is outside 1..=12"),
            )),
        }
    }

    fn number(self) -> u8 {
        match self {
            Self::Manual(slot) => slot,
            Self::Autosave => 0,
        }
    }
}

/// Save slot metadata available without loading it into the player.
#[derive(Debug)]
pub enum SaveSlotState {
    Empty,
    Compatible(SaveFile),
    Incompatible(SaveFile, SaveValidationError),
    Corrupt(String),
}

/// Resolves the per-project save directory using the platform data directory.
pub fn project_save_dir(project_id: &str) -> io::Result<PathBuf> {
    let root = if cfg!(target_os = "windows") {
        env::var_os("APPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join("Library/Application Support"))
    } else {
        env::var_os("XDG_DATA_HOME").map(PathBuf::from).or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join(".local/share"))
        })
    }
    .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "OS data directory unavailable"))?;
    Ok(root.join("vinyl").join(project_id).join("saves"))
}

/// Atomically writes a save slot without replacing the previous file until serialization succeeds.
pub fn write_save(directory: &Path, slot: SaveSlot, save: &SaveFile) -> io::Result<PathBuf> {
    fs::create_dir_all(directory)?;
    let path = directory.join(slot.file_name()?);
    let temporary = path.with_extension("json.tmp");
    {
        let file = File::create(&temporary)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, save).map_err(io::Error::other)?;
        writer.flush()?;
        writer.get_ref().sync_all()?;
    }
    fs::rename(&temporary, &path)?;
    Ok(path)
}

/// Reads a save slot without applying compatibility policy.
pub fn read_save(directory: &Path, slot: SaveSlot) -> io::Result<SaveFile> {
    let file = File::open(directory.join(slot.file_name()?))?;
    serde_json::from_reader(BufReader::new(file)).map_err(io::Error::other)
}

/// Inspects a slot while retaining incompatible or corrupt files for display and overwrite.
pub fn inspect_save(
    directory: &Path,
    slot: SaveSlot,
    project_id: &ProjectId,
    project_version: &str,
    script_hash: &str,
) -> io::Result<SaveSlotState> {
    let path = directory.join(slot.file_name()?);
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(SaveSlotState::Empty),
        Err(error) => return Err(error),
    };
    let save = match serde_json::from_reader(BufReader::new(file)) {
        Ok(save) => save,
        Err(error) => return Ok(SaveSlotState::Corrupt(error.to_string())),
    };
    Ok(
        match validate_save(&save, project_id, project_version, script_hash) {
            Ok(()) => SaveSlotState::Compatible(save),
            Err(error) => SaveSlotState::Incompatible(save, error),
        },
    )
}
