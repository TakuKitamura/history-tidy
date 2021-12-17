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
    let mut stdout: std::io::Stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    let backend: tui::backend::TermionBackend<std::io::Stdout> = TermionBackend::new(stdout);
    let mut terminal: tui::Terminal<tui::backend::TermionBackend<std::io::Stdout>> =
        Terminal::new(backend).unwrap();

    let mut app: App = App::new(map);
    // app.next();
    app.state.select(Some(0));
    let res: String = run_app(&mut terminal, app);

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
    let mut script_file: std::fs::File = match std::fs::File::create(script_path) {
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
    header_cells: Vec<&'static str>,
}

impl App {
    fn new(history_map: HashMap<String, HashMap<String, String>>) -> App {
        let mut hashtags: Vec<Vec<String>> = vec![];
        for hashtag in (&history_map).keys() {
            let item_count: usize = history_map.get(hashtag).unwrap().len();
            hashtags.push(vec![hashtag.to_string(), item_count.to_string()]);
        }

        // println!("{:?}", hashtags);

        App {
            state: TableState::default(),
            table_title: " Select Hashtag View ",
            hashtags,
            history_map,
            header_cells: vec!["HashTag", "Item Count"],
        }
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> String {
    loop {
        terminal
            .draw(|f: &mut tui::Frame<B>| ui(f, &mut app))
            .unwrap();

        if let Event::Key(key) = event::read().unwrap() {
            let key_code: crossterm::event::KeyCode = key.code;
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
            } else if app.table_title == " Select Hashtag View "
                && (key_code == KeyCode::Enter || key_code == KeyCode::Right)
            {
                let selected: usize = app.state.selected().unwrap();
                let item: &Vec<String> = &app.hashtags[selected];
                let history_group: &&HashMap<String, String> =
                    &app.history_map.get(item[0].as_str()).unwrap();

                let mut hashtags: Vec<Vec<String>> = vec![];
                for (history, message) in history_group.iter() {
                    hashtags.push(vec![history.to_string(), message.to_string()]);
                }
                app.header_cells = vec!["Command", "Comment"];
                app.hashtags = hashtags;
                app.state.select(Some(0));
                app.table_title = " Select Command View ";
            } else if app.table_title == " Select Command View " && key_code == KeyCode::Enter {
                let selected: usize = app.state.selected().unwrap();
                let item: &Vec<String> = &app.hashtags[selected];
                return item[0].to_owned();
            } else if app.table_title == " Select Command View " && key_code == KeyCode::Left {
                let mut hashtags: Vec<Vec<String>> = vec![];
                for hashtag in (&app.history_map).keys() {
                    let item_count: usize = app.history_map.get(hashtag).unwrap().len();
                    hashtags.push(vec![hashtag.to_string(), item_count.to_string()]);
                }
                app.hashtags = hashtags;
                app.header_cells = vec!["HashTag", "Item Count"];
                app.state.select(Some(0));
                app.table_title = " Select Hashtag View ";
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

    let selected_style: tui::style::Style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style: tui::style::Style = Style::default().bg(Color::Blue);
    let header_cells: std::iter::Map<std::slice::Iter<&str>, _> = app
        .header_cells
        .iter()
        .map(|h: &&str| Cell::from(&(**h)).style(Style::default().fg(Color::Red)));
    let header: tui::widgets::Row = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows: std::iter::Map<std::slice::Iter<Vec<String>>, _> =
        app.hashtags.iter().map(|item: &Vec<String>| {
            let height: usize = item
                .iter()
                .map(|content: &String| content.chars().filter(|c: &char| *c == '\n').count())
                .max()
                .unwrap_or(0)
                + 1;
            let cells: std::iter::Map<std::slice::Iter<std::string::String>, _> =
                item.iter().map(|content: &String| content.to_string());
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
