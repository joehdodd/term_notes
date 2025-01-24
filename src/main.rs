use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{DefaultTerminal, Frame};
use serde::{Deserialize, Serialize};
//use std::fs::OpenOptions;
use std::io::Result;
//use tui_input::Input;

//  enum InputMode {
//      Normal,
//      Insert,
//  }

#[derive(Clone, Serialize, Deserialize)]
struct Note {
    title: String,
    body: String,
    date_created: String,
}

#[warn(dead_code)]
pub struct App {
    exit: bool,
}

// TODO
// 1. Update Todo struct to have detail key whose value is String
// 2. Update app layout to be two panes vertically and horizontally
//      1. Top pane has list of todos on the left, detail view for todo on the right
//      2. Bottom pane has input for todos and details on left, help on right
// 3. Update app logic to handle inputting for list and for details
fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App { exit: false };
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget("Hello, World!", frame.area());
    }

    fn handle_events(&mut self) -> Result<()> {
        match crossterm::event::read()? {
            Event::Key(key_event) => self.handle_key_events(key_event)?,
            _ => {}
        }
        Ok(())
    }

    fn handle_key_events(&mut self, event: KeyEvent) -> Result<()> {
        match event.code {
            KeyCode::Char('q') => self.exit = true,
            _ => {}
        }
        Ok(())
    }
}
