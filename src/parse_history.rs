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
use linked_hash_map::LinkedHashMap;
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

pub fn get_command_hashmap() -> LinkedHashMap<String, LinkedHashMap<String, String>> {
    let history_vec: Vec<String> = match get_tidy_history() {
        Ok(history_vec) => history_vec,
        Err(e) => {
            println!("{}", e);
            vec![]
        }
    };

    let mut command_hashmap: LinkedHashMap<String, LinkedHashMap<String, String>> =
        LinkedHashMap::new();
    for history in &history_vec {
        let lexer: Lexer<Chars> = Lexer::new(history.chars());
        let mut parser: Parser<Lexer<Chars>, DefaultBuilder<String>> = DefaultParser::new(lexer);

        match parser.and_or_list() {
            Ok(_) => {
                let new_line: Vec<Newline> = parser.linebreak();
                if new_line.is_empty() {
                    let text: String = "ALL".to_owned();
                    if command_hashmap.contains_key(&text) == false {
                        let mut map_hashtag: LinkedHashMap<String, String> = LinkedHashMap::new();
                        map_hashtag.insert(history.to_owned(), "".to_owned());
                        command_hashmap.insert(text, map_hashtag);
                    } else {
                        let map_hashtag: &mut LinkedHashMap<String, String> =
                            command_hashmap.get_mut(&text).unwrap();
                        // if map_hashtag.contains_key(history) == false {
                        map_hashtag.insert(history.to_owned(), "".to_owned());
                        // }
                    }
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
                            let mut map_hashtag: LinkedHashMap<String, String> =
                                LinkedHashMap::new();
                            map_hashtag.insert(history.to_owned(), message.to_owned());
                            command_hashmap.insert(text, map_hashtag);
                        } else {
                            let map_hashtag: &mut LinkedHashMap<String, String> =
                                command_hashmap.get_mut(&text).unwrap();
                            if map_hashtag.contains_key(history) && message.is_empty() {
                                message = map_hashtag.get_mut(history).unwrap().to_owned();
                            }
                            map_hashtag.insert(history.to_owned(), message.to_owned());
                        }
                    }
                }
            }
            Err(_) => {
                let text: String = "ERR".to_owned();
                if command_hashmap.contains_key(&text) == false {
                    let mut map_hashtag: LinkedHashMap<String, String> = LinkedHashMap::new();
                    map_hashtag.insert(history.to_owned(), "".to_owned());
                    command_hashmap.insert(text, map_hashtag);
                } else {
                    let map_hashtag: &mut LinkedHashMap<String, String> =
                        command_hashmap.get_mut(&text).unwrap();
                    if map_hashtag.contains_key(history) == false {
                        map_hashtag.insert(history.to_owned(), "".to_owned());
                    }
                }
            }
        }
    }
    return command_hashmap;
}
