use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dirs::home_dir;
use linked_hash_map::LinkedHashMap;
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

const SELECT_HASHTAG_TITLE: &'static str = " Select Hashtag View ";
const SELECT_COMMAND_TITLE: &'static str = " Select Command View ";

pub fn init_ui(map: LinkedHashMap<String, LinkedHashMap<String, String>>) {
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
    history_map: LinkedHashMap<String, LinkedHashMap<String, String>>,
    header_cells: Vec<&'static str>,
    select_hashtag_header: Vec<&'static str>,
    select_command_header: Vec<&'static str>,
}

impl App {
    fn new(history_map: LinkedHashMap<String, LinkedHashMap<String, String>>) -> App {
        let mut hashtags: Vec<Vec<String>> = vec![];
        for hashtag in (&history_map).keys() {
            let item_count: usize = history_map.get(hashtag).unwrap().len();
            hashtags.push(vec![hashtag.to_owned(), item_count.to_string()]);
        }

        let select_hashtag_header: Vec<&'static str> = vec!["HashTag", "Item Count"];
        let select_command_header: Vec<&'static str> = vec!["Command", "Comment"];

        hashtags.sort();

        App {
            state: TableState::default(),
            table_title: SELECT_HASHTAG_TITLE,
            hashtags,
            history_map,
            header_cells: select_hashtag_header.to_owned(),
            select_hashtag_header: select_hashtag_header.to_owned(),
            select_command_header: select_command_header.to_owned(),
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
                let select_hashtag: &Vec<String> = &app.hashtags[selected];
                // println!("{}", select_hashtag[0]);
                let history_group: &&LinkedHashMap<String, String> =
                    &app.history_map.get(select_hashtag[0].as_str()).unwrap();

                let mut hashtags: Vec<Vec<String>> = vec![];
                for (history, message) in history_group.iter() {
                    if select_hashtag[0] == "ALL" {
                        hashtags.push(vec![history.to_owned()]);
                    } else {
                        hashtags.push(vec![history.to_owned(), message.to_owned()]);
                    }
                }
                if select_hashtag[0] == "ALL" {
                    app.header_cells = vec!["Command"]
                } else {
                    app.header_cells = app.select_command_header.to_owned();
                }
                hashtags.reverse();
                app.hashtags = hashtags;
                app.state.select(Some(0));
                app.table_title = SELECT_COMMAND_TITLE;
            } else if app.table_title == SELECT_COMMAND_TITLE && key_code == KeyCode::Enter {
                let selected: usize = app.state.selected().unwrap();
                let select_command: &Vec<String> = &app.hashtags[selected];
                return select_command[0].to_owned();
            } else if app.table_title == SELECT_COMMAND_TITLE && key_code == KeyCode::Left {
                let mut hashtags: Vec<Vec<String>> = vec![];
                for hashtag in (&app.history_map).keys() {
                    let item_count: usize = app.history_map.get(hashtag).unwrap().len();
                    hashtags.push(vec![hashtag.to_owned(), item_count.to_string()]);
                }
                hashtags.sort();
                app.hashtags = hashtags;
                app.header_cells = app.select_hashtag_header.to_owned();
                app.state.select(Some(0));
                app.table_title = SELECT_HASHTAG_TITLE;
            }
        }
    }
}

fn generate_wrapped_text(text: String, limit: u32, sepalate: &str) -> String {
    let mut chars: Vec<&str> = text.split("").collect();
    chars.remove(0);
    chars.remove(chars.len() - 1);
    let mut converted_chars: Vec<String> = vec![];

    let mut width_count: u32 = 0;
    while chars.len() > 0 && limit >= 2 {
        let c: &str = &chars[0];
        if c.bytes().len() == 1 {
            width_count += 1;
        } else {
            width_count += 2;
        }

        if width_count == limit + 1 {
            converted_chars.push(sepalate.to_owned());
            width_count = 0;
        } else {
            converted_chars.push(c.to_owned());
            if width_count > limit - 1 && chars.len() > 1 {
                converted_chars.push(sepalate.to_owned());
                width_count = 0;
            }
            chars.remove(0);
        }
    }
    return converted_chars.join("");
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let fsize: tui::layout::Rect = f.size();
    let rects_margin: u16 = 2;
    let highlight_symbol: &str = "> ";
    let rects: Vec<tui::layout::Rect> = Layout::default()
        .horizontal_margin(rects_margin)
        .vertical_margin(rects_margin)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(fsize);

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
        let mut height_count: u16 = 1;
        let border_on: u32 = 2;
        let cells: Map<Iter<String>, _> = item.iter().map(|content: &String| {
            let converted_string = generate_wrapped_text(
                content.to_owned(),
                fsize.width as u32
                    - border_on
                    - (highlight_symbol.len() as u32)
                    - (rects_margin * 2) as u32,
                "\n",
            );
            height_count = converted_string.matches("\n").count() as u16 + 1;
            return converted_string;
        });

        Row::new(cells).height(height_count as u16)
    });
    let header_cells_count: u16 = 100 / (app.header_cells.len() as u16);
    let widths: &[tui::layout::Constraint; 3] = &[
        Constraint::Percentage(header_cells_count),
        Constraint::Length(30),
        Constraint::Min(10),
    ];
    let t: tui::widgets::Table = Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(app.table_title),
        )
        .highlight_style(selected_style)
        .highlight_symbol(highlight_symbol)
        .widths(widths);
    f.render_stateful_widget(t, rects[0], &mut app.state);
}
