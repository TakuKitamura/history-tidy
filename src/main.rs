use clap::{crate_authors, crate_description, crate_version};
use clap::{Arg, SubCommand};
use conch_parser::lexer::Lexer;
use conch_parser::parse::DefaultParser;
use dirs;
use std::collections::HashMap;
mod hashtag;
mod ui;
use hashtag::HashtagParser;
use std::fs;
use std::io::Write;

// get history vector with duplicates removed
fn get_tidy_history() -> Result<Vec<String>, std::io::Error> {
    match dirs::home_dir() {
        Some(mut history_file_path) => {
            history_file_path.push(".history-tidy");
            history_file_path.push("history");
            match std::fs::read_to_string(history_file_path) {
                Ok(history_file_content) => {
                    let history_vec: Vec<String> = history_file_content
                        .lines()
                        .map(|line: &str| line.to_string().trim().to_string())
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

fn main() {
    let matches: clap::ArgMatches = clap::App::new("history-tidy")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand(SubCommand::with_name("load").about("load command selected"))
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

    if let Some(matches) = matches.subcommand_matches("init") {
        let shell: &str = matches.value_of("shell-type").unwrap();
        let init_shell_script: &str = match shell {
            "bash" => include_str!("../bin/init.bash"),
            _ => unreachable!(),
        };
        println!("{}", init_shell_script);
        std::process::exit(0);
    }

    if let Some(_) = matches.subcommand_matches("load") {
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
        let script_content: String = match std::fs::read_to_string(&script_path) {
            Ok(script_content) => script_content,
            Err(e) => {
                println!("{}", e);
                return;
            }
        };

        let mut script_file: fs::File = match fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&script_path)
        {
            Ok(script_file) => script_file,
            Err(e) => {
                println!("{}", e);
                return;
            }
        };
        let script_file_content: String = String::new();
        match script_file.write_all(script_file_content.as_bytes()) {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e);
                return;
            }
        };
        println!("{}", script_content);
        std::process::exit(0);
    }

    let mut map: HashMap<String, HashMap<String, String>> = HashMap::new();
    let history_vec: Vec<String> = match get_tidy_history() {
        Ok(history_vec) => history_vec,
        Err(e) => {
            println!("{}", e);
            vec![]
        }
    };

    for history in &history_vec {
        let lexer: conch_parser::lexer::Lexer<std::str::Chars> = Lexer::new(history.chars());
        let mut parser: conch_parser::parse::Parser<
            conch_parser::lexer::Lexer<std::str::Chars>,
            conch_parser::ast::builder::DefaultBuilder<String>,
        > = DefaultParser::new(lexer);

        match parser.and_or_list() {
            Ok(_) => {
                let new_line: Vec<conch_parser::ast::builder::Newline> = parser.linebreak();
                if new_line.is_empty() {
                } else {
                    let hashtags_str: String = new_line[0].0.as_ref().unwrap().to_owned();
                    let history: &String = &history
                        .replace(hashtags_str.as_str(), "")
                        .trim()
                        .to_string();

                    let hashtags: Vec<hashtag::Hashtag> =
                        HashtagParser::new(&hashtags_str).collect::<Vec<hashtag::Hashtag>>();

                    let end: usize = hashtags[hashtags.len() - 1].end;

                    let mut message: String = "".to_owned();
                    for s in hashtags_str.char_indices() {
                        let (i, c): (usize, char) = s;
                        if i > end {
                            message += c.to_string().as_str();
                        }
                    }
                    message = message.trim().to_string();

                    for hashtag in hashtags {
                        let text: String = format!("#{}", hashtag.text.to_string().to_owned());
                        if map.contains_key(&text) == false {
                            let mut map_hashtag: HashMap<String, String> = HashMap::new();
                            map_hashtag.insert(history.to_string(), message.to_string());
                            map.insert(
                                text, // tag
                                map_hashtag,
                            );
                        } else {
                            let map_hashtag: &mut HashMap<String, String> =
                                map.get_mut(&text).unwrap();
                            if map_hashtag.contains_key(history) == false {
                                map_hashtag.insert(history.to_string(), message.to_string());
                            }
                            let map_history: &mut String = map_hashtag.get_mut(history).unwrap();
                            if message.to_string().len() != 0 {
                                *map_history = message.to_string();
                            }
                        }
                    }
                }
            }
            Err(_) => {}
        }
    }

    // println!("{:?}", map);
    // println!("{:?}", history_vec);
    ui::init_ui(map);
    std::process::exit(0);
}
