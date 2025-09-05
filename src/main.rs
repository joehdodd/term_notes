use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;
use ratatui::Terminal;
use std::io;
use std::io::Result;

mod notes_app;
use crate::notes_app::App;
use crate::notes_app::NotesRepository;

fn main() -> Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let repo = NotesRepository::new("notes.json");

    // instantiate app
    let mut app = App::new(repo.load_notes()?, repo);
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
