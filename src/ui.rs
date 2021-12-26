use crossterm::{
    event::{self, Event, KeyCode},
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
use textwrap::word_separators::*;
use textwrap::word_splitters::*;
use textwrap::Options;
use tui::{
    backend::{Backend, TermionBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};

use unicode_width::UnicodeWidthStr;

const SELECT_HASHTAG_TITLE: &'static str = " Select Hashtag View ";
const SELECT_COMMAND_TITLE: &'static str = " Select Command View ";
const HASHTAG_VIEW_ID: u8 = 1;
const ALL_COMMAND_VIEW_ID: u8 = 2;
const HASHTAG_COMMAND_VIEW_ID: u8 = 3;

const ALL_HASHTAG: &'static str = "ALL";

const WRAP_TABLE_TEXT: &str = "table";
const WRAP_EDITOR_TEXT: &str = "editor";

pub fn init_ui(map: linked_hash_map::LinkedHashMap<String, Vec<String>>) {
    enable_raw_mode().unwrap();
    let mut stdout: Stdout = stdout();
    execute!(stdout, EnterAlternateScreen).unwrap();
    let backend: tui::backend::TermionBackend<Stdout> = TermionBackend::new(stdout);
    let mut terminal: Terminal<tui::backend::TermionBackend<Stdout>> =
        Terminal::new(backend).unwrap();

    let mut app: App = App::new(map);
    app.state.select(Some(0));
    let res: String = run_app(&mut terminal, app);

    disable_raw_mode().unwrap();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
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
    hashtags_memo: Vec<Vec<String>>,
    history_map: LinkedHashMap<String, Vec<String>>,
    header_cells: Vec<String>,
    select_hashtag_header: Vec<String>,
    view_id: u8,
    input: String,
    input_mode: bool,
    scroll: u16,
    debug: String,
}

impl App {
    fn new(history_map: LinkedHashMap<String, Vec<String>>) -> App {
        let mut hashtags: Vec<Vec<String>> = vec![];
        let mut all_hashtag: Vec<String> = vec![];
        for hashtag in (&history_map).keys() {
            let item_count: usize = history_map.get(hashtag).unwrap().len();
            if hashtag == ALL_HASHTAG {
                all_hashtag = vec![hashtag.to_owned(), item_count.to_string()];
            } else {
                hashtags.push(vec![hashtag.to_owned(), item_count.to_string()]);
            }
        }

        hashtags.sort();
        hashtags.insert(0, all_hashtag);

        let hashtags_memo: Vec<Vec<String>> = hashtags.clone();

        let select_hashtag_header: Vec<String> = vec!["HashTag".to_owned(), "Count".to_owned()];

        App {
            state: TableState::default(),
            table_title: SELECT_HASHTAG_TITLE,
            hashtags,
            hashtags_memo: hashtags_memo,
            history_map,
            header_cells: select_hashtag_header.to_owned(),
            select_hashtag_header: select_hashtag_header.to_owned(),
            view_id: HASHTAG_VIEW_ID,
            input: String::new(),
            input_mode: false,
            scroll: 0,
            debug: String::new(),
        }
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> String {
    loop {
        terminal.draw(|f: &mut Frame<B>| ui(f, &mut app)).unwrap();

        if let Event::Key(key) = event::read().unwrap() {
            let key_code: event::KeyCode = key.code;

            if app.input_mode {
                if key_code == KeyCode::Enter {
                    // app.messages.push(app.input.drain(..).collect());
                    app.debug = format!("{}", app.input);
                    app.input_mode = false;
                } else if key_code == KeyCode::Backspace {
                    if app.input.len() > 0 {
                        let x = app.input.pop().unwrap();
                        if x == '\n' {
                            app.input.pop();
                            if app.scroll > 0 {
                                app.scroll -= 1;
                            }
                        }
                    }
                } else if key_code == KeyCode::Esc {
                    app.input_mode = false;
                } else {
                    match key_code {
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        _ => {}
                    }
                }

                continue;
            }

            if key_code == KeyCode::Char('q') {
                return "".to_owned();
            } else if key_code == KeyCode::Char('e') {
                app.input_mode = true;
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
                let history_group: &&Vec<String> =
                    &app.history_map.get(select_hashtag[0].as_str()).unwrap();

                let mut hashtags: Vec<Vec<String>> = vec![];
                for history in history_group.iter() {
                    if select_hashtag[0] == ALL_HASHTAG {
                        hashtags.push(vec![history.to_owned()]);
                    } else {
                        hashtags.push(vec![history.to_owned()]);
                    }
                }

                app.header_cells = vec![select_hashtag[0].clone()];
                if select_hashtag[0] == ALL_HASHTAG {
                    app.view_id = ALL_COMMAND_VIEW_ID;
                } else {
                    app.view_id = HASHTAG_COMMAND_VIEW_ID;
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
                app.hashtags = app.hashtags_memo.clone();
                app.header_cells = app.select_hashtag_header.to_owned();
                app.state.select(Some(0));
                app.table_title = SELECT_HASHTAG_TITLE;
                app.view_id = HASHTAG_VIEW_ID;
            }
        }
    }
}
fn wrap_text(text: String, limit: usize, mode: &str) -> String {
    let base_options = Options::new(limit).word_splitter(NoHyphenation);
    if mode == WRAP_TABLE_TEXT {
        let options = base_options.word_separator(UnicodeBreakProperties);
        return textwrap::fill(text.as_str(), &options);
    } else if mode == WRAP_EDITOR_TEXT {
        let options = base_options.word_separator(AsciiSpace);
        return textwrap::fill(text.as_str(), &options);
    } else {
        unreachable!();
    }
}

fn ui<B: Backend>(frame: &mut Frame<B>, app: &mut App) {
    let frame_size: tui::layout::Rect = frame.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(frame_size.height - 2),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(frame.size());

    let highlight_symbol: &str = "> ";
    let normal_style: Style = Style::default()
        .add_modifier(Modifier::UNDERLINED)
        .add_modifier(Modifier::BOLD);
    let header_cells: Map<Iter<String>, _> = app.header_cells.iter().map(|h| Cell::from(&(**h)));
    let header: tui::widgets::Row = Row::new(header_cells).height(1).style(normal_style);

    let rows: Map<Iter<Vec<String>>, _> = app.hashtags.iter().map(|item| {
        let text_margin: usize = highlight_symbol.len();

        let text_width: usize = if chunks[0].width as usize >= text_margin {
            chunks[0].width as usize - text_margin
        } else {
            chunks[0].width as usize
        };

        let mut height_count: u16 = 1;
        if app.view_id == ALL_COMMAND_VIEW_ID || app.view_id == HASHTAG_COMMAND_VIEW_ID {
            // one line
            let cells: Map<Iter<String>, _> = item.iter().map(|content: &String| {
                let content: String = "$ ".to_owned() + content.as_str();
                let converted_string = wrap_text(content, text_width, WRAP_TABLE_TEXT);
                height_count = converted_string.matches("\n").count() as u16 + 1;
                return converted_string;
            });

            return Row::new(cells).height(height_count as u16);
        } else {
            // two line
            let cells: Map<Iter<String>, _> = item.iter().map(|content: &String| {
                let converted_string =
                    wrap_text(content.to_owned(), text_width / 2, WRAP_TABLE_TEXT);

                let tmp_height_count = converted_string.matches("\n").count() as u16 + 1;
                if tmp_height_count > height_count {
                    height_count = tmp_height_count;
                }

                return converted_string;
            });

            return Row::new(cells).height(height_count as u16);
        };
    });
    let header_cells_count: u16 = 100 / (app.header_cells.len() as u16);
    let widths: &[tui::layout::Constraint; 2] = &[
        Constraint::Percentage(header_cells_count),
        Constraint::Percentage(header_cells_count),
    ];
    let selected_style: Style = Style::default().add_modifier(Modifier::REVERSED);
    let table: tui::widgets::Table = Table::new(rows)
        .header(header)
        .highlight_style(selected_style)
        .highlight_symbol(highlight_symbol)
        .widths(widths);

    if app.input_mode == false {
        frame.render_stateful_widget(table, chunks[0], &mut app.state);
    } else {
        // ウィンドウサイズを変更しないかぎり普遍
        let chunks_width: usize = chunks[0].width as usize;
        let chunks_height: usize = chunks[0].height as usize;

        let raw_input = app.input.replace("\n", "");

        app.input = wrap_text(raw_input, chunks_width as usize, WRAP_EDITOR_TEXT);

        let return_count: usize = app.input.matches("\n").count();
        let last_line_width: usize = app.input.split("\n").last().unwrap().width();

        app.scroll = if return_count < chunks_height {
            0
        } else {
            return_count as u16 - chunks_height as u16 + 1
        } + if last_line_width == chunks_width && chunks_height <= return_count + 1 {
            1
        } else {
            0
        };

        let input = Paragraph::new(app.input.as_ref()).scroll((app.scroll, 0));
        frame.render_widget(input, chunks[0]);

        let cursor_x: u16 = if last_line_width == chunks_width {
            0
        } else {
            last_line_width as u16
        };

        let cursor_y: u16 = if last_line_width == chunks_width {
            return_count as u16 + 1
        } else {
            return_count as u16
        } - app.scroll;

        frame.set_cursor(cursor_x, cursor_y)
    }

    let text = vec![
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("Select", Style::default().fg(Color::Green)),
            Span::raw(": Arrow Keys and Enter Key"),
        ]),
        Spans::from(vec![
            Span::raw("  "),
            Span::styled("Quit", Style::default().fg(Color::Green)),
            Span::raw(": 'q' Key, "),
            Span::styled(app.debug.as_str(), Style::default().fg(Color::Red)),
        ]),
    ];
    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, chunks[1]);
}
