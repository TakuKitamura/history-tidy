use crate::hashtag::Hashtag;
use crate::hashtag::HashtagParser;
use dirs::home_dir;
use std::fs::read_to_string;
use std::io::Error;
use std::io::ErrorKind;

use builder::DefaultBuilder;
use builder::Newline;

use conch_parser::ast::builder;
use conch_parser::lexer::Lexer;
use conch_parser::parse::DefaultParser;
use conch_parser::parse::Parser;

use std::collections::HashMap;

use std::str::Chars;

fn get_tidy_history() -> Result<Vec<String>, Error> {
    match home_dir() {
        Some(mut history_file_path) => {
            history_file_path.push(".history-tidy");
            history_file_path.push("history");
            match read_to_string(history_file_path) {
                Ok(history_file_content) => {
                    let history_vec: Vec<String> = history_file_content
                        .lines()
                        .map(|line: &str| line.to_owned().trim().to_owned())
                        .collect();
                    return Ok(history_vec);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        None => {
            return Err(Error::new(ErrorKind::NotFound, "Can't get home path"));
        }
    };
}

pub fn get_command_hashmap() -> HashMap<String, HashMap<String, String>> {
    let history_vec: Vec<String> = match get_tidy_history() {
        Ok(history_vec) => history_vec,
        Err(e) => {
            println!("{}", e);
            vec![]
        }
    };

    let mut command_hashmap: HashMap<String, HashMap<String, String>> = HashMap::new();
    for history in &history_vec {
        let lexer: Lexer<Chars> = Lexer::new(history.chars());
        let mut parser: Parser<Lexer<Chars>, DefaultBuilder<String>> = DefaultParser::new(lexer);

        match parser.and_or_list() {
            Ok(_) => {
                let new_line: Vec<Newline> = parser.linebreak();
                if new_line.is_empty() {
                } else {
                    let hashtags_str: String = new_line[0].0.as_ref().unwrap().to_owned();
                    let history: &String =
                        &history.replace(hashtags_str.as_str(), "").trim().to_owned();

                    let hashtags: Vec<Hashtag> =
                        HashtagParser::new(&hashtags_str).collect::<Vec<Hashtag>>();

                    let end: usize = hashtags[hashtags.len() - 1].end;

                    let mut message: String = "".to_owned();
                    for s in hashtags_str.char_indices() {
                        let (i, c): (usize, char) = s;
                        if i > end {
                            message += c.to_string().as_str();
                        }
                    }
                    message = message.trim().to_owned();

                    for hashtag in hashtags {
                        let text: String = format!("#{}", hashtag.text.to_owned().to_owned());
                        if command_hashmap.contains_key(&text) == false {
                            let mut map_hashtag: HashMap<String, String> = HashMap::new();
                            map_hashtag.insert(history.to_owned(), message.to_owned());
                            command_hashmap.insert(text, map_hashtag);
                        } else {
                            let map_hashtag: &mut HashMap<String, String> =
                                command_hashmap.get_mut(&text).unwrap();
                            if map_hashtag.contains_key(history) == false {
                                map_hashtag.insert(history.to_owned(), message.to_owned());
                            }
                            let map_history: &mut String = map_hashtag.get_mut(history).unwrap();
                            if message.to_owned().len() != 0 {
                                *map_history = message.to_owned();
                            }
                        }
                    }
                }
            }
            Err(_) => {}
        }
    }
    return command_hashmap;
}
