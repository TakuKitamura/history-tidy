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

// enum InputMode {
//     Normal,
//     Editing,
// }

const SELECT_HASHTAG_TITLE: &'static str = " Select Hashtag View ";
const SELECT_COMMAND_TITLE: &'static str = " Select Command View ";
const HASHTAG_VIEW_ID: u8 = 1;
const ALL_COMMAND_VIEW_ID: u8 = 2;
const HASHTAG_COMMAND_VIEW_ID: u8 = 3;

const ALL_HASHTAG: &'static str = "ALL";

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
    show_popup: bool,
    input: String,
    input_mode: bool,
    messages: Vec<String>,
    scroll: u16,
    debug: String,
    last_width: usize,
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
            show_popup: false,
            input: String::new(),
            input_mode: false,
            messages: Vec::new(),
            scroll: 0,
            debug: String::new(),
            last_width: 0,
        }
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> String {
    loop {
        terminal.draw(|f: &mut Frame<B>| ui(f, &mut app)).unwrap();

        if let Event::Key(key) = event::read().unwrap() {
            let key_code: event::KeyCode = key.code;

            // match app.input_mode {
            // InputMode::Normal => match key.code {
            //     KeyCode::Char('e') => {
            //         app.input_mode = InputMode::Editing;
            //     }
            //     // KeyCode::Char('q') => {
            //     //     return Ok(());
            //     // }
            //     _ => {}
            // },

            // }

            if app.input_mode {
                if key_code == KeyCode::Enter {
                    // app.messages.push(app.input.drain(..).collect());
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
                    // let x = app.input.pop().unwrap();
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

                app.last_width = app.input.split("\n").last().unwrap().width();
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

#[derive(Clone, Copy, Debug)]
pub struct MyWordSplitter;

/// `Hello` implements `WordSplitter` by not splitting the
/// word at all.
impl WordSplitter for MyWordSplitter {
    fn split_points(&self, word: &str) -> Vec<usize> {
        // let x = word.char_indices().map(|(x, y)| word.len()).collect();
        return vec![];
    }
}

// impl WordSeparator for MyWordSplitter {
//     fn is_word_separator(&self, c: char) -> bool {
//         c.is_whitespace()
//     }
// }

fn generate_wrapped_text(text: String, limit: u32, mode: &str) -> String {
    if mode == "NoHyphenation" {
        let options = Options::new(limit as usize).word_splitter(NoHyphenation);
        return textwrap::fill(text.as_str(), &options);
    } else {
        let options = Options::new(limit as usize)
            .word_splitter(NoHyphenation)
            .word_separator(AsciiSpace);
        // options;
        return textwrap::fill(text.as_str(), &options);
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
        let text_margin: u32 = highlight_symbol.len() as u32;

        let text_width: u32 = if chunks[0].width as u32 >= text_margin {
            chunks[0].width as u32 - text_margin
        } else {
            chunks[0].width as u32
        };

        let mut height_count: u16 = 1;
        if app.view_id == ALL_COMMAND_VIEW_ID || app.view_id == HASHTAG_COMMAND_VIEW_ID {
            // one line
            let cells: Map<Iter<String>, _> = item.iter().map(|content: &String| {
                let content: String = "$ ".to_owned() + content.as_str();
                let converted_string = generate_wrapped_text(content, text_width, "NoHyphenation");
                height_count = converted_string.matches("\n").count() as u16 + 1;
                return converted_string;
            });

            return Row::new(cells).height(height_count as u16);
        } else {
            // two line
            let cells: Map<Iter<String>, _> = item.iter().map(|content: &String| {
                let converted_string =
                    generate_wrapped_text(content.to_owned(), text_width / 2, "NoHyphenation");

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
    let widths: &[tui::layout::Constraint; 3] = &[
        Constraint::Percentage(header_cells_count),
        Constraint::Length(30),
        Constraint::Min(10),
    ];
    let selected_style: Style = Style::default().add_modifier(Modifier::REVERSED);
    let table: tui::widgets::Table = Table::new(rows)
        .header(header)
        .highlight_style(selected_style)
        .highlight_symbol(highlight_symbol)
        .widths(widths);
    // frame.render_stateful_widget(table, reacts[0], &mut app.state);

    // if app.show_popup {
    // let block = Block::default().title("Popup").borders(Borders::ALL);
    // let area = centered_rect(60, 20, frame_size);
    // frame.render_widget(Clear, area); //this clears out the background
    // frame.render_widget(block, area);
    if app.input_mode == false {
        frame.render_stateful_widget(table, chunks[0], &mut app.state);
    } else {
        // ウィンドウサイズを変更しないかぎり普遍
        let chunks_width = chunks[0].width;
        let chunks_height = chunks[0].height;

        // println!("{}", chunks_height);
        // app.input = generate_wrapped_text(app.input.to_owned(), chunks_width);

        let app_input = app.input.to_owned();

        app.input = generate_wrapped_text(app_input.replace("\n", ""), chunks_width as u32, "HOGE");

        let last_width = app_input.split("\n").last().unwrap().width();
        let count = app.input.to_owned().matches("\n").count();
        // app.debug = format!("{}, {}", app_input.width(), chunks_width);
        if last_width > chunks_width as usize {
            // if last_width == chunks_width as usize {
            //     app.input += "\n";
            // }

            let line_count = app.input.to_owned().matches("\n").count() + 1;
            if line_count > chunks_height as usize {
                app.scroll += 1;
            }
        }

        let input = Paragraph::new(app.input.as_ref()).scroll((app.scroll, 0));
        frame.render_widget(input, chunks[0]);

        let mut x = last_width as u16;

        // let count = app_input.match_indices("\n").count();
        let mut y = count as u16 - app.scroll;

        if x == chunks_width {
            x = 0;
            y += 1;
        }
        frame.set_cursor(x, y)
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

// fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
//     let popup_layout = Layout::default()
//         .direction(Direction::Vertical)
//         .constraints(
//             [
//                 Constraint::Percentage((100 - percent_y) / 2),
//                 Constraint::Percentage(percent_y),
//                 Constraint::Percentage((100 - percent_y) / 2),
//             ]
//             .as_ref(),
//         )
//         .split(r);

//     Layout::default()
//         .direction(Direction::Horizontal)
//         .constraints(
//             [
//                 Constraint::Percentage((100 - percent_x) / 2),
//                 Constraint::Percentage(percent_x),
//                 Constraint::Percentage((100 - percent_x) / 2),
//             ]
//             .as_ref(),
//         )
//         .split(popup_layout[1])[1]
// }
