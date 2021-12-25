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

pub fn get_tidy_history() -> Result<Vec<String>, Error> {
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

pub fn get_command_hashmap(
    history_vec: Vec<String>,
) -> LinkedHashMap<String, LinkedHashMap<String, String>> {
    let mut command_hashmap: LinkedHashMap<String, LinkedHashMap<String, String>> =
        LinkedHashMap::new();
    let mut all: LinkedHashMap<String, String> = LinkedHashMap::new();
    for history in &history_vec {
        if history.len() == 0 {
            continue;
        }
        let lexer: Lexer<Chars> = Lexer::new(history.chars());
        let mut parser: Parser<Lexer<Chars>, DefaultBuilder<String>> = DefaultParser::new(lexer);

        all.insert(history.to_owned(), "".to_owned());

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

                    let mut message: String = "".to_owned();

                    if hashtags.len() > 0 {
                        let end: usize = hashtags[hashtags.len() - 1].end;

                        for s in hashtags_str.char_indices() {
                            let (i, c): (usize, char) = s;
                            if i > end {
                                message += c.to_string().as_str();
                            }
                        }
                        message = message.trim().to_owned();
                    }

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
                // let text: String = "ERR".to_owned();
                // if command_hashmap.contains_key(&text) == false {
                //     let mut map_hashtag: LinkedHashMap<String, String> = LinkedHashMap::new();
                //     map_hashtag.insert(history.to_owned(), "".to_owned());
                //     command_hashmap.insert(text, map_hashtag);
                // } else {
                //     let map_hashtag: &mut LinkedHashMap<String, String> =
                //         command_hashmap.get_mut(&text).unwrap();
                //     if map_hashtag.contains_key(history) == false {
                //         map_hashtag.insert(history.to_owned(), "".to_owned());
                //     }
                // }
            }
        }
    }
    command_hashmap.insert("ALL".to_owned(), all);
    return command_hashmap;
}

#[cfg(test)]
fn test_get_tidy_history() {
    let history_vec: Vec<String> = get_tidy_history().unwrap();
    assert_eq!(history_vec.len(), 0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_tidy_history_test() {
        let history: Vec<String> = vec!["ls -a", "pwd #hoge", "cd ~ #hoge #fuga", "ls -a"]
            .iter()
            .map(|s: &&str| s.to_string())
            .collect();
        let command_hashmap = get_command_hashmap(history);

        let expected_command_hashmap = vec![
            ("#hoge", vec![("pwd", ""), ("cd ~", "")]),
            ("#fuga", vec![("cd ~", "")]),
            (
                "ALL",
                vec![("pwd #hoge", ""), ("cd ~ #hoge #fuga", ""), ("ls -a", "")],
            ),
        ]
        .into_iter()
        .map(|(k1, v1)| {
            (
                k1.to_owned(),
                v1.into_iter()
                    .map(|(k2, v2)| (k2.to_owned(), v2.to_owned()))
                    .collect::<Vec<(String, String)>>()
                    .into_iter()
                    .collect::<LinkedHashMap<String, String>>(),
            )
        })
        .collect::<LinkedHashMap<String, LinkedHashMap<String, String>>>();

        assert_eq!(command_hashmap, expected_command_hashmap);
    }
}
