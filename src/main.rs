use clap::{crate_authors, crate_description, crate_version};
use clap::{Arg, SubCommand};
use conch_parser::lexer::Lexer;
use conch_parser::parse::DefaultParser;
use dirs;
use std::collections::HashMap;
mod hashtag;
use duct_sh;
use hashtag::HashtagParser;
use std::os::unix::process::CommandExt;
use std::process::Command;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tui::{
    backend::{Backend, TermionBackend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame, Terminal,
};

// get history vector with duplicates removed
fn get_tidy_history() -> Result<Vec<String>, std::io::Error> {
    match dirs::home_dir() {
        Some(mut history_file_path) => {
            history_file_path.push(".history-tidy");
            match std::fs::read_to_string(history_file_path) {
                Ok(history_file_content) => {
                    let history_vec: Vec<String> = history_file_content
                        .lines()
                        .map(|line| line.to_string().trim().to_string())
                        .collect();
                    return Ok(history_vec);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Can't get home path",
            ));
        }
    };
}

// #[derive(Debug)]
// struct History {
//     history: Vec<String>,
//     message: String,
// }

fn main() {
    // let matches = clap::App::new("history-tidy")
    //     .version(crate_version!())
    //     .author(crate_authors!())
    //     .about(crate_description!())
    //     .subcommand(SubCommand::with_name("list").about("listing history with tags"))
    //     .subcommand(
    //         SubCommand::with_name("init")
    //             .about("print the shell script to execute history-tidy")
    //             .arg(
    //                 Arg::with_name("shell-type")
    //                     .possible_values(&["bash"])
    //                     .required(true),
    //             ),
    //     )
    //     .get_matches();

    let mut map: HashMap<String, HashMap<String, String>> = HashMap::new();
    match get_tidy_history() {
        Ok(history_vec) => {
            // tag: command: message

            for history in &history_vec {
                let lexer = Lexer::new(history.chars());
                let mut parser = DefaultParser::new(lexer);

                match parser.and_or_list() {
                    Ok(ast) => {
                        let new_line = parser.linebreak();
                        if new_line.is_empty() {
                        } else {
                            let hashtags_str = new_line[0].0.as_ref().unwrap().to_owned();
                            let history = &history
                                .replace(hashtags_str.as_str(), "")
                                .trim()
                                .to_string();

                            let hashtags = HashtagParser::new(&hashtags_str).collect::<Vec<_>>();

                            let end = hashtags[hashtags.len() - 1].end;

                            let mut message = "".to_owned();
                            for s in hashtags_str.char_indices() {
                                let (i, c) = s;
                                if i > end {
                                    message += c.to_string().as_str();
                                }
                            }
                            message = message.trim().to_string();
                            // println!("message:{}", message);

                            for hashtag in hashtags {
                                let text = format!("#{}", hashtag.text.to_string().to_owned());
                                if map.contains_key(&text) == false {
                                    let mut map_hashtag: HashMap<String, String> = HashMap::new();
                                    map_hashtag.insert(history.to_string(), message.to_string());
                                    map.insert(
                                        text, // tag
                                        map_hashtag,
                                    );
                                } else {
                                    let map_hashtag = map.get_mut(&text).unwrap();
                                    if map_hashtag.contains_key(history) == false {
                                        map_hashtag
                                            .insert(history.to_string(), message.to_string());
                                    }
                                    let map_history = map_hashtag.get_mut(history).unwrap();
                                    if message.to_string().len() != 0 {
                                        *map_history = message.to_string();
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {}
                }
            }

            // println!("{:?}", map);
            // println!("{:?}", history_vec);

            enable_raw_mode().unwrap();
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
            let backend = TermionBackend::new(stdout);
            let mut terminal = Terminal::new(backend).unwrap();

            let mut app = App::new(map, history_vec);
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

            if let Err(err) = res {
                eprintln!("{:?}", err)
            }
        }
        Err(e) => {
            eprintln!("Failed to load the file: {}", e);
            std::process::exit(1);
        }
    }
    std::process::exit(0);
}

struct App {
    state: TableState,
    hashtags: Vec<Vec<String>>,
    history_map: HashMap<String, HashMap<String, String>>,
    history: Vec<String>,
    header_cells: Vec<String>,
}

impl App {
    fn new(history_map: HashMap<String, HashMap<String, String>>, history: Vec<String>) -> App {
        let mut hashtags = vec![];
        for hashtag in (&history_map).keys() {
            let item_count = history_map.get(hashtag).unwrap().len();
            hashtags.push(vec![hashtag.to_string(), item_count.to_string()]);
        }

        // println!("{:?}", hashtags);

        App {
            state: TableState::default(),
            hashtags: hashtags,
            history_map,
            history: history,
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let mut state = 0;
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
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
                        return Ok(());
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
