use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::{
    layout::Layout,
    widgets::{Block, Borders, List, ListDirection, ListState, Paragraph, Wrap},
    DefaultTerminal, Frame,
};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::Result;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

enum InputMode {
    Normal,
    Insert,
}

#[derive(Debug)]
enum CurrentScreen {
    Notes,
    Edit,
}

#[derive(Clone, Serialize, Deserialize)]
struct Note {
    title: String,
    body: String,
    date_created: String,
}

#[derive(Debug)]
struct Cursor {
    x: u16,
    y: u16,
}

pub struct App {
    exit: bool,
    notes: Vec<Note>,
    input: String,
    input_mode: InputMode,
    screen: CurrentScreen,
    cursor: Cursor,
    character_index: usize,
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
        input: String::new(),
        input_mode: InputMode::Normal,
        screen: CurrentScreen::Notes,
        cursor: Cursor { x: 0, y: 0 },
        character_index: 0,
    };
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let mut list_state = ListState::default();
        list_state.select_first();
        while !self.exit {
            terminal.draw(|frame| self.draw(frame, &mut list_state))?;
            self.handle_events(&mut list_state)?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, list_state: &mut ListState) {
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
        let selected = self.notes.get(list_state.selected().unwrap_or(0)).unwrap();
        let paragraph = match self.input_mode {
            InputMode::Normal => Paragraph::new(selected.body.to_owned())
                .block(Block::new().borders(Borders::ALL))
                .wrap(Wrap { trim: true }),
            InputMode::Insert => Paragraph::new(self.input.to_owned())
                .block(Block::new().borders(Borders::ALL))
                .wrap(Wrap { trim: true }),
        };
        match self.screen {
            CurrentScreen::Notes => {}
            CurrentScreen::Edit => {
                frame.set_cursor_position(Position {
                    x: layout_horizontal[1].x + self.cursor.x + 1,
                    y: layout_horizontal[1].y + self.cursor.y + 1,
                });
            }
        }
        frame.render_stateful_widget(list, layout_horizontal[0], list_state);
        frame.render_widget(paragraph, layout_horizontal[1]);
    }

    fn handle_events(&mut self, list_state: &mut ListState) -> Result<()> {
        match crossterm::event::read()? {
            Event::Key(key_event) => self.handle_key_events(key_event, list_state)?,
            _ => {}
        }
        Ok(())
    }

    fn handle_key_events(&mut self, event: KeyEvent, list_state: &mut ListState) -> Result<()> {
        match self.screen {
            CurrentScreen::Notes => self.handle_notes_key_events(event, list_state)?,
            CurrentScreen::Edit => match self.input_mode {
                InputMode::Normal => self.handle_edit_key_events(event, list_state)?,
                InputMode::Insert => self.handle_insert_key_events(event)?,
            },
        }
        Ok(())
    }

    fn handle_notes_key_events(
        &mut self,
        event: KeyEvent,
        list_state: &mut ListState,
    ) -> Result<()> {
        match event.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('j') => list_state.select_next(),
            KeyCode::Char('k') => list_state.select_previous(),
            KeyCode::Tab => self.screen = CurrentScreen::Edit,
            _ => {}
        }
        Ok(())
    }

    fn handle_edit_key_events(
        &mut self,
        event: KeyEvent,
        list_state: &mut ListState,
    ) -> Result<()> {
        match event.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('j') => {
                self.cursor = Cursor {
                    x: self.cursor.x,
                    y: self.cursor.y + 1,
                };
            }
            KeyCode::Char('k') => {
                self.cursor = Cursor {
                    x: self.cursor.x,
                    y: self.cursor.y - if self.cursor.y > 0 { 1 } else { 0 },
                };
            }
            KeyCode::Char('h') => {
                self.cursor = Cursor {
                    x: self.cursor.x - if self.cursor.x > 0 { 1 } else { 0 },
                    y: self.cursor.y,
                };
            }
            KeyCode::Char('l') => {
                self.cursor = Cursor {
                    x: self.cursor.x + 1,
                    y: self.cursor.y,
                };
            }
            KeyCode::Char('i') => {
                let selected_note = self.notes.get(list_state.selected().unwrap_or(0)).unwrap();
                self.input = selected_note.body.to_owned();
                self.input_mode = InputMode::Insert;
            }
            KeyCode::Tab => self.screen = CurrentScreen::Notes,
            _ => {}
        }
        Ok(())
    }

    fn handle_insert_key_events(&mut self, event: KeyEvent) -> Result<()> {
        match event.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }
}
