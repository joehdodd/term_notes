use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;
use ratatui::Terminal;
use serde_json::from_str;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::io::Result;

use crate::notes_app::notes_app::{App, JsonNote};
pub mod notes_app;

fn main() -> Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // retrieve file
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("notes.json");
    let mut contents = String::new();
    match file {
        Ok(ref mut file) => {
            file.read_to_string(&mut contents)?;
        }
        Err(e) => {
            eprintln!("Error with file: {}", e);
        }
    }

    // parse json file to Vec<JsonNote>
    let file_json: Vec<JsonNote> = if contents.is_empty() {
        Vec::new()
    } else {
        from_str(&contents)?
    };

    // instantiate app
    let mut app = App::new(file_json);
    // if let Err(e) captures the error and prints it
    // if app.run fails
    if let Err(e) = app.run(&mut terminal) {
        eprintln!("Error: {}", e);
    }

    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
