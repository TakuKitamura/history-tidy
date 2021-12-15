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

fn main() {
    let matches = clap::App::new("history-tidy")
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
        let shell = matches.value_of("shell-type").unwrap();
        let init_shell_script = match shell {
            "bash" => include_str!("../bin/init.bash"),
            _ => unreachable!(),
        };
        println!("{}", init_shell_script);
        std::process::exit(0);
    }

    if let Some(matches) = matches.subcommand_matches("load") {
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
        let script_content = match std::fs::read_to_string(&script_path) {
            Ok(script_content) => script_content,
            Err(e) => {
                println!("{}", e);
                return;
            }
        };
        // write empty strring to script file
        let mut script_file = match fs::OpenOptions::new()
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
        let script_file_content = String::new();
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
    let history_vec = match get_tidy_history() {
        Ok(history_vec) => history_vec,
        Err(e) => {
            println!("{}", e);
            // return;
            vec![]
        }
    };

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
                                map_hashtag.insert(history.to_string(), message.to_string());
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
    ui::init_ui(map);
    std::process::exit(0);
}
