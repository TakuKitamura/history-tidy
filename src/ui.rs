use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::collections::HashMap;
use std::io;
use std::io::Write;
use tui::{
    backend::{Backend, TermionBackend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame, Terminal,
};

pub fn init_ui(map: HashMap<String, HashMap<String, String>>) {
    enable_raw_mode().unwrap();
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut app = App::new(map);
    // app.next();
    app.state.select(Some(0));
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode().unwrap();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .unwrap();
    terminal.show_cursor().unwrap();

    let script_path: std::path::PathBuf = match dirs::home_dir() {
        Some(mut history_file_path) => {
            history_file_path.push(".history-tidy");
            history_file_path.push("script");
            history_file_path
        }
        None => {
            return;
        }
    };
    // write res in script
    let mut script_file = match std::fs::File::create(script_path) {
        Ok(file) => file,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };
    match script_file.write_all(res.as_bytes()) {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
            return;
        }
    }
}

struct App {
    state: TableState,
    hashtags: Vec<Vec<String>>,
    history_map: HashMap<String, HashMap<String, String>>,
    header_cells: Vec<String>,
}

impl App {
    fn new(history_map: HashMap<String, HashMap<String, String>>) -> App {
        let mut hashtags = vec![];
        for hashtag in (&history_map).keys() {
            let item_count = history_map.get(hashtag).unwrap().len();
            hashtags.push(vec![hashtag.to_string(), item_count.to_string()]);
        }

        // println!("{:?}", hashtags);

        App {
            state: TableState::default(),
            hashtags,
            history_map,
            header_cells: vec!["HashTag".to_string(), "Item Count".to_string()],
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.hashtags.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.hashtags.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> String {
    let mut state = 0;
    loop {
        terminal.draw(|f| ui(f, &mut app)).unwrap();

        if let Event::Key(key) = event::read().unwrap() {
            match key.code {
                KeyCode::Char('q') => return "".to_owned(),
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
                KeyCode::Enter => {
                    if state == 0 {
                        let selected = app.state.selected().unwrap();
                        let item = &app.hashtags[selected];
                        let history_group = &app.history_map.get(item[0].as_str()).unwrap();
                        app.header_cells = vec!["Command".to_string(), "Comment".to_string()];
                        let mut hashtags = vec![];
                        for (history, message) in history_group.iter() {
                            hashtags.push(vec![history.to_string(), message.to_string()]);
                        }
                        app.hashtags = hashtags;
                        app.state.select(Some(0));
                        state = 1;
                    } else {
                        let selected = app.state.selected().unwrap();
                        let item = &app.hashtags[selected];
                        // println!("{}", item[0]);
                        return item[0].to_owned();
                    }
                }
                KeyCode::Right => {
                    if state == 0 {
                        let selected = app.state.selected().unwrap();
                        let item = &app.hashtags[selected];
                        let history_group = &app.history_map.get(item[0].as_str()).unwrap();
                        app.header_cells = vec!["Command".to_string(), "Comment".to_string()];
                        let mut hashtags = vec![];
                        for (history, message) in history_group.iter() {
                            hashtags.push(vec![history.to_string(), message.to_string()]);
                        }
                        app.hashtags = hashtags;
                        app.state.select(Some(0));
                        state = 1;
                    }
                }
                KeyCode::Left => {
                    if state == 1 {
                        app.header_cells = vec!["HashTag".to_string(), "Item Count".to_string()];
                        app.hashtags = vec![vec![
                            app.hashtags[0][0].to_string(),
                            app.hashtags[0][1].to_string(),
                        ]];
                        state = 0;

                        let mut hashtags = vec![];
                        for hashtag in (&app.history_map).keys() {
                            let item_count = app.history_map.get(hashtag).unwrap().len();
                            hashtags.push(vec![hashtag.to_string(), item_count.to_string()]);
                        }
                        app.hashtags = hashtags;
                        app.state.select(Some(0));
                        app.header_cells = vec!["HashTag".to_string(), "Item Count".to_string()];
                    }
                }
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let rects = Layout::default()
        .horizontal_margin(3)
        .vertical_margin(3)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);
    let xxx: Vec<&str> = app.header_cells.iter().map(|s| &**s).collect();
    let header_cells = xxx
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows = app.hashtags.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|content| content.to_string());
        Row::new(cells).height(height as u16)
    });
    let t = Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" History Tags "),
        )
        .highlight_style(selected_style)
        .highlight_symbol("> ")
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Length(30),
            Constraint::Min(10),
        ]);
    f.render_stateful_widget(t, rects[0], &mut app.state);
}
