use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct JsonNote {
    pub title: String,
    pub body: String,
    pub date_created: String,
}
mod notes_app;
pub use self::notes_app::App;
mod notes_repository;
pub use self::notes_repository::NotesRepository;
