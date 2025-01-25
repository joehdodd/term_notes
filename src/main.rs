use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::{
    layout::Layout,
    widgets::{Block, Borders, List, ListDirection, ListItem, ListState, Paragraph, Wrap},
    DefaultTerminal, Frame,
};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::Result;
use tui_input::Input;

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

pub struct App {
    exit: bool,
    notes: Vec<Note>,
    input: Input,
}

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("notes.json")
        .expect("Could not open or create file.");
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let file_json: Vec<Note> = if contents.is_empty() {
        Vec::new()
    } else {
        serde_json::from_str(&contents).expect("Could not process file.")
    };
    let mut app = App {
        exit: false,
        notes: file_json.to_vec(),
        input: Input::default(),
    };
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let mut list_state = ListState::default();
        list_state.select_first();
        //let first_note = self.notes.get(0);
        //self.input.with_value(first_note.unwrap().body.to_owned());
        while !self.exit {
            terminal.draw(|frame| self.draw(frame, &mut list_state))?;
            self.handle_events(&mut list_state)?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame, list_state: &mut ListState) {
        // let layout_vertical = Layout::default()
        //     .direction(Direction::Vertical)
        //     .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        //     .split(frame.area());
        let layout_horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(frame.area());
        let items: Vec<String> = self
            .notes
            .iter()
            .map(|item| item.title.to_owned())
            .collect();
        let list = List::new(items)
            .block(Block::new().borders(Borders::ALL))
            .highlight_style(Style::new().black().bg(Color::Green))
            .direction(ListDirection::TopToBottom);
        frame.render_stateful_widget(list, layout_horizontal[0], list_state);
        let selected = self.notes.get(list_state.selected().unwrap_or(0)).unwrap();
        frame.render_widget(
            Paragraph::new(selected.body.to_owned())
                .block(Block::new().borders(Borders::ALL))
                .wrap(Wrap { trim: true }),
            layout_horizontal[1],
        );
    }

    fn handle_events(&mut self, list_state: &mut ListState) -> Result<()> {
        match crossterm::event::read()? {
            Event::Key(key_event) => self.handle_key_events(key_event, list_state)?,
            _ => {}
        }
        Ok(())
    }

    fn handle_key_events(&mut self, event: KeyEvent, list_state: &mut ListState) -> Result<()> {
        match event.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('j') => list_state.select_next(),
            KeyCode::Char('k') => list_state.select_previous(),
            _ => {}
        }
        Ok(())
    }
}
