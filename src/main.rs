use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;
use ratatui::{
    layout::Layout,
    widgets::{Block, Borders, List, ListDirection, ListState, Padding},
    Frame, Terminal,
};
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::io::Result;
use tui_textarea::{CursorMove, TextArea};

enum InputMode {
    Normal,
    Insert,
}

#[derive(Debug)]
enum CurrentScreen {
    List,
    Edit,
}

#[derive(Clone, Serialize, Deserialize)]
struct JsonNote {
    title: String,
    body: String,
    date_created: String,
}

#[derive(Clone)]
struct Note<'a> {
    title: String,
    body: TextArea<'a>,
}

pub struct App<'a> {
    exit: bool,
    notes: Vec<Note<'a>>,
    current_note: usize,
    input_mode: InputMode,
    screen: CurrentScreen,
}

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
        .open("notes.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

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

impl App<'_> {
    fn new(file_json: Vec<JsonNote>) -> App<'static> {
        let app_notes: Vec<Note> = file_json
            .iter()
            .map(|note| {
                let mut text_area =
                    TextArea::new(note.body.split("\n").map(|line| line.to_string()).collect());
                text_area.set_block(Block::default().borders(Borders::ALL));
                Note {
                    title: note.title.to_owned(),
                    body: text_area,
                }
            })
            .collect();
        App {
            exit: false,
            notes: app_notes,
            current_note: 0,
            input_mode: InputMode::Normal,
            screen: CurrentScreen::List,
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let mut list_state = ListState::default();
        list_state.select_first();
        while !self.exit {
            terminal.draw(|frame| self.draw(frame, &mut list_state))?;
            self.handle_events(&mut list_state)?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, list_state: &mut ListState) {
        self.current_note = list_state.selected().unwrap_or(0);

        let layout_horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(frame.area());
        let items: Vec<String> = self
            .notes
            .iter()
            .map(|item| item.title.to_owned())
            .collect();

        let (list_style, cursor_style, cursor_line_style) = match self.screen {
            CurrentScreen::List => (
                Style::new().black().bg(Color::Green),
                Style::default(),
                Style::default(),
            ),
            CurrentScreen::Edit => match self.input_mode {
                InputMode::Normal => (
                    Style::default().bg(Color::White).fg(Color::Black),
                    Style::default().bg(Color::White).fg(Color::Black),
                    Style::default(),
                ),
                InputMode::Insert => (
                    Style::default().bg(Color::White).fg(Color::Black),
                    Style::default().bg(Color::Green).fg(Color::Black),
                    Style::default(),
                ),
            },
        };

        let list = List::new(items)
            .block(Block::new().borders(Borders::ALL))
            .highlight_style(list_style)
            .direction(ListDirection::TopToBottom);
        let selected: &mut Note<'_> = self
            .notes
            .get_mut(list_state.selected().unwrap_or(0))
            .unwrap();
        selected.body.set_cursor_style(cursor_style);
        selected.body.set_cursor_line_style(cursor_line_style);
        frame.render_widget(&selected.body, layout_horizontal[1]);
        frame.render_stateful_widget(list, layout_horizontal[0], list_state);
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
            CurrentScreen::List => self.handle_list_key_events(event, list_state)?,
            CurrentScreen::Edit => match self.input_mode {
                InputMode::Normal => self.handle_edit_key_events(event, list_state)?,
                InputMode::Insert => self.handle_insert_key_events(event)?,
            },
        }
        Ok(())
    }

    fn handle_list_key_events(
        &mut self,
        event: KeyEvent,
        list_state: &mut ListState,
    ) -> Result<()> {
        match event.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('j') => {
                let selected = match list_state.selected() {
                    Some(selected) => selected,
                    None => 0,
                };
                if selected == self.notes.len() - 1 {
                    list_state.select_first();
                } else {
                    list_state.select_next();
                }
            }
            KeyCode::Char('k') => list_state.select_previous(),
            KeyCode::Tab => self.screen = CurrentScreen::Edit,
            _ => {}
        }
        Ok(())
    }

    fn handle_edit_key_events(
        &mut self,
        event: KeyEvent,
        _list_state: &mut ListState,
    ) -> Result<()> {
        let selected_note = self.notes.get_mut(self.current_note).unwrap();
        match event.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('j') => selected_note.body.move_cursor(CursorMove::Down),
            KeyCode::Char('k') => selected_note.body.move_cursor(CursorMove::Up),
            KeyCode::Char('h') => selected_note.body.move_cursor(CursorMove::Back),
            KeyCode::Char('l') => selected_note.body.move_cursor(CursorMove::Forward),
            KeyCode::Char('i') => self.input_mode = InputMode::Insert,
            KeyCode::Tab => self.screen = CurrentScreen::List,
            _ => {}
        }
        Ok(())
    }

    fn handle_insert_key_events(&mut self, event: KeyEvent) -> Result<()> {
        let selected_note = self.notes.get_mut(self.current_note).unwrap();
        match event.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
            }
            _ => {
                // Call in input on body of currently selected note
                selected_note.body.input(event);
            }
        }
        Ok(())
    }
}
