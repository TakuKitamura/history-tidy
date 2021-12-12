use conch_parser::lexer::Lexer;
use conch_parser::parse::DefaultParser;
use dirs;
use std::collections::HashMap;
mod hashtag;
use hashtag::HashtagParser;

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
    match get_tidy_history() {
        Ok(history_vec) => {
            // tag: command: message
            let mut map: HashMap<String, HashMap<String, String>> = HashMap::new();
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

            for (tag, map_hashtag) in &map {
                println!("tag:{}", tag);
                // for (history, message) in map_hashtag {
                //     println!("history:{}", history);
                //     println!("message:{}", message);
                // }
            }

            // for (tag, map_hashtag) in &map {
            //     let x = map.get_key_value("#abc");
            for (_, message) in map.get_key_value("#abc") {
                for (y, z) in message {
                    // get history index in enmuulate history_vec
                    let mut i = 1;
                    for h in &history_vec {
                        if h.trim() == y.trim() {
                            println!("#abc:id:{}", i);
                            break;
                        }
                        i += 1;
                    }

                    println!("#abc:history:{}", y);
                    println!("#abc:message:{}", z);
                }
                // println!("history:{}", history);
                // println!("message:{:?}", message);
            }
            // }
        }
        Err(e) => {
            eprintln!("Failed to load the file: {}", e);
            std::process::exit(1);
        }
    }
    std::process::exit(0);
}
