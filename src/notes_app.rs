pub mod notes_app {
    use crossterm::event::{Event, KeyCode, KeyEvent};
    use ratatui::prelude::*;
    use ratatui::{
        layout::Layout,
        widgets::{
            Block, Borders, Clear, List, ListDirection, ListState, Padding, Paragraph, Wrap,
        },
        Frame, Terminal,
    };
    use serde::{Deserialize, Serialize};
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
        Help,
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub struct JsonNote {
        pub title: String,
        pub body: String,
        pub date_created: String,
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

    impl App<'_> {
        pub fn new(file_json: Vec<JsonNote>) -> App<'static> {
            let app_notes: Vec<Note> = file_json
                .iter()
                .map(|note| {
                    let mut text_area =
                        TextArea::new(note.body.split("\n").map(|line| line.to_string()).collect());
                    text_area.set_block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title_bottom(" ?: toggle help ")
                            .title_alignment(Alignment::Center),
                    );
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
                _ => (Style::default(), Style::default(), Style::default()),
            };

            let list = List::new(items)
                .block(Block::new().borders(Borders::ALL))
                .highlight_style(list_style)
                .direction(ListDirection::TopToBottom);
            match self.notes.get_mut(self.current_note) {
                Some(note) => {
                    let selected = note;
                    let title = Paragraph::new(selected.title.clone())
                        .block(Block::default().borders(Borders::ALL).title(" Term_Notes "))
                        .alignment(Alignment::Center);
                    selected.body.set_cursor_style(cursor_style);
                    selected.body.set_cursor_line_style(cursor_line_style);
                    frame.render_widget(title, layout_horizontal[1]);
                }
                None => {
                    let title = Paragraph::new("No notes")
                        .block(Block::default().borders(Borders::ALL).title(" Term_Notes "))
                        .alignment(Alignment::Center);
                    frame.render_widget(title, layout_horizontal[1]);
                }
            }
            frame.render_stateful_widget(list, layout_horizontal[0], list_state);

            if let CurrentScreen::Help = self.screen {
                self.render_popup(frame).unwrap();
            }
        }

        fn render_popup(&mut self, frame: &mut Frame) -> Result<()> {
            let popup_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Percentage(25),
                    Constraint::Percentage(50),
                    Constraint::Percentage(25),
                ])
                .split(frame.area());
            let vertical_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                    Constraint::Percentage(25),
                    Constraint::Percentage(50),
                    Constraint::Percentage(25),
                ])
                .split(popup_layout[1]);
            let help_paragrpah = Paragraph::new(vec![
                "j: move cursor down".into(),
                "k: move cursor up".into(),
                "h: move cursor left".into(),
                "l: move cursor right".into(),
                "i: insert mode".into(),
                "esc: normal mode".into(),
                "?: toggle help".into(),
                "q: quit".into(),
                "tab: switch screens".into(),
            ])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Term_Notes Help ")
                    .title_alignment(Alignment::Center)
                    .padding(Padding::uniform(1)),
            )
            .wrap(Wrap { trim: true });
            frame.render_widget(Clear, vertical_layout[1]);
            frame.render_widget(help_paragrpah, vertical_layout[1]);
            Ok(())
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
                CurrentScreen::Help => {
                    if event.code == KeyCode::Char('?') {
                        self.screen = CurrentScreen::Edit;
                    }
                }
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
                KeyCode::Char('?') => self.screen = CurrentScreen::Help,
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
                KeyCode::Char('?') => self.screen = CurrentScreen::Help,
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
}
