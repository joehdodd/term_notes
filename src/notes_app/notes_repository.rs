use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Result};
use crate::notes_app::JsonNote;

pub struct NotesRepository {
    pub path: String,
}

impl NotesRepository {
    pub fn new(path: &str) -> Self {
        Self { path: path.to_string() }
    }

    pub fn load_notes(&self) -> Result<Vec<JsonNote>> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        if contents.trim().is_empty() {
            Ok(Vec::new())
        } else {
            Ok(serde_json::from_str(&contents)?)
        }
    }

    pub fn save_notes(&self, notes: &[JsonNote]) -> Result<()> {
        let json = serde_json::to_string_pretty(notes)?;
        let mut file = File::create(&self.path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}