use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dirs::home_dir;
use std::collections::HashMap;
use std::fs::File;
use std::io::stdout;
use std::io::Stdout;
use std::io::Write;
use std::iter::Map;
use std::path::PathBuf;
use std::slice::Iter;
use tui::{
    backend::{Backend, TermionBackend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame, Terminal,
};

const SELECT_HASHTAG_HEADER: [&'static str; 2] = ["HashTag", "Item Count"];
const SELECT_COMMAND_HEADER: [&'static str; 2] = ["Command", "Comment"];

const SELECT_HASHTAG_TITLE: &'static str = " Select Hashtag View ";
const SELECT_COMMAND_TITLE: &'static str = " Select Command View ";

pub fn init_ui(map: HashMap<String, HashMap<String, String>>) {
    enable_raw_mode().unwrap();
    let mut stdout: Stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    let backend: tui::backend::TermionBackend<Stdout> = TermionBackend::new(stdout);
    let mut terminal: Terminal<tui::backend::TermionBackend<Stdout>> =
        Terminal::new(backend).unwrap();

    let mut app: App = App::new(map);
    app.state.select(Some(0));
    let res: String = run_app(&mut terminal, app);

    disable_raw_mode().unwrap();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .unwrap();
    terminal.show_cursor().unwrap();

    let script_path: PathBuf = match home_dir() {
        Some(mut history_file_path) => {
            history_file_path.push(".history-tidy");
            history_file_path.push("script");
            history_file_path
        }
        None => {
            return;
        }
    };

    let mut script_file: File = match File::create(script_path) {
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
    table_title: &'static str,
    hashtags: Vec<Vec<String>>,
    history_map: HashMap<String, HashMap<String, String>>,
    header_cells: [&'static str; 2],
}

impl App {
    fn new(history_map: HashMap<String, HashMap<String, String>>) -> App {
        let mut hashtags: Vec<Vec<String>> = vec![];
        for hashtag in (&history_map).keys() {
            let item_count: usize = history_map.get(hashtag).unwrap().len();
            hashtags.push(vec![hashtag.to_owned(), item_count.to_string()]);
        }

        App {
            state: TableState::default(),
            table_title: SELECT_HASHTAG_TITLE,
            hashtags,
            history_map,
            header_cells: SELECT_HASHTAG_HEADER,
        }
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> String {
    loop {
        terminal.draw(|f: &mut Frame<B>| ui(f, &mut app)).unwrap();

        if let Event::Key(key) = event::read().unwrap() {
            let key_code: event::KeyCode = key.code;
            if key_code == KeyCode::Char('q') {
                return "".to_owned();
            } else if key_code == KeyCode::Down {
                let i: usize = match app.state.selected() {
                    Some(i) => {
                        if i >= app.hashtags.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                app.state.select(Some(i));
            } else if key_code == KeyCode::Up {
                let i: usize = match app.state.selected() {
                    Some(i) => {
                        if i == 0 {
                            app.hashtags.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                app.state.select(Some(i));
            } else if app.table_title == SELECT_HASHTAG_TITLE
                && (key_code == KeyCode::Enter || key_code == KeyCode::Right)
            {
                let selected: usize = app.state.selected().unwrap();
                let item: &Vec<String> = &app.hashtags[selected];
                let history_group: &&HashMap<String, String> =
                    &app.history_map.get(item[0].as_str()).unwrap();

                let mut hashtags: Vec<Vec<String>> = vec![];
                for (history, message) in history_group.iter() {
                    hashtags.push(vec![history.to_owned(), message.to_owned()]);
                }
                app.header_cells = SELECT_COMMAND_HEADER;
                app.hashtags = hashtags;
                app.state.select(Some(0));
                app.table_title = SELECT_COMMAND_TITLE;
            } else if app.table_title == SELECT_COMMAND_TITLE && key_code == KeyCode::Enter {
                let selected: usize = app.state.selected().unwrap();
                let item: &Vec<String> = &app.hashtags[selected];
                return item[0].to_owned();
            } else if app.table_title == SELECT_COMMAND_TITLE && key_code == KeyCode::Left {
                let mut hashtags: Vec<Vec<String>> = vec![];
                for hashtag in (&app.history_map).keys() {
                    let item_count: usize = app.history_map.get(hashtag).unwrap().len();
                    hashtags.push(vec![hashtag.to_owned(), item_count.to_string()]);
                }
                app.hashtags = hashtags;
                app.header_cells = SELECT_HASHTAG_HEADER;
                app.state.select(Some(0));
                app.table_title = SELECT_HASHTAG_TITLE;
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let rects: Vec<tui::layout::Rect> = Layout::default()
        .horizontal_margin(3)
        .vertical_margin(3)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let selected_style: Style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style: Style = Style::default().bg(Color::Blue);
    let header_cells: Map<Iter<&str>, _> = app
        .header_cells
        .iter()
        .map(|h: &&str| Cell::from(&(**h)).style(Style::default().fg(Color::Red)));
    let header: tui::widgets::Row = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows: Map<Iter<Vec<String>>, _> = app.hashtags.iter().map(|item: &Vec<String>| {
        let height: usize = item
            .iter()
            .map(|content: &String| content.chars().filter(|c: &char| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells: Map<Iter<String>, _> = item.iter().map(|content: &String| content.to_owned());
        Row::new(cells).height(height as u16)
    });
    let t: tui::widgets::Table = Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(app.table_title),
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
