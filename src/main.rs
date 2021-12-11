use clap::{crate_authors, crate_description, crate_version};
use clap::{Arg, SubCommand};
use dirs;

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
                    let mut history_vec: Vec<String> = history_file_content
                        .lines()
                        .map(|line| line.to_string().trim().to_string())
                        .collect();
                    history_vec.sort();
                    history_vec.dedup();
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

struct App<'a> {
    state: TableState,
    items: Vec<Vec<&'a str>>,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            state: TableState::default(),
            items: vec![
                vec!["Row11"],
                vec!["Row21"],
                vec!["Row31"],
                vec!["Row41"],
                vec!["Row51"],
                vec!["Row61"],
                vec!["Row71"],
                vec!["Row81"],
                vec!["Row91"],
                vec!["Row101"],
                vec!["Row111"],
                vec!["Row121"],
                vec!["Row131"],
                vec!["Row141"],
                vec!["Row151"],
                vec!["Row161"],
                vec!["Row171"],
                vec!["Row181"],
                vec!["Row191"],
            ],
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
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
                    self.items.len() - 1
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
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
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
    let header_cells = ["Tags"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows = app.items.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(*c));
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
            Constraint::Percentage(100),
            Constraint::Length(30),
            Constraint::Min(10),
        ]);
    f.render_stateful_widget(t, rects[0], &mut app.state);
}

fn main() {
    let matches = clap::App::new("history-tidy")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand(SubCommand::with_name("list").about("listing history with tags"))
        .subcommand(
            SubCommand::with_name("init")
                .about("print the shell script to execute history-tidy")
                .arg(
                    Arg::with_name("shell-type")
                        .possible_values(&["bash"])
                        .required(true),
                ),
        )
        .get_matches();

    // no args or subcommand is 'init' case
    if let Some(matches) = matches.subcommand_matches("init") {
        let shell = matches.value_of("shell-type").unwrap();
        let init_shell_script = match shell {
            "bash" => include_str!("../bin/init.bash"),
            _ => unreachable!(),
        };
        println!("{}", init_shell_script);
        std::process::exit(0);
    }

    // no args or subcommand is 'list' case
    match get_tidy_history() {
        Ok(history_vec) => {
            // for line in history_vec {
            //     println!("{}", line);
            // }

            enable_raw_mode().unwrap();
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
            let backend = TermionBackend::new(stdout);
            let mut terminal = Terminal::new(backend).unwrap();

            let mut app = App::new();
            app.next();
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
