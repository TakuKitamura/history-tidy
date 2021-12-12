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

#[derive(Debug)]
struct History {
    history: Vec<String>,
    message: String,
}

fn main() {
    match get_tidy_history() {
        Ok(history_vec) => {
            let mut map: HashMap<String, History> = HashMap::new();
            for history in history_vec {
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
                            println!("message:{}", message);

                            for hashtag in hashtags {
                                let text = format!("#{}", hashtag.text.to_string().to_owned());
                                if map.contains_key(&text) == false {
                                    map.insert(
                                        text,
                                        History {
                                            history: vec![(&history).to_string()],
                                            message: message.to_owned(),
                                        },
                                    );
                                } else {
                                    if map
                                        .get_mut(&text)
                                        .unwrap()
                                        .history
                                        .iter()
                                        .any(|h| h == history)
                                        == false
                                    {
                                        map.get_mut(&text)
                                            .unwrap()
                                            .history
                                            .push((&history).to_string());
                                    }
                                    if message.is_empty() == false {
                                        map.get_mut(&text).unwrap().message = message.to_owned();
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {}
                }
            }
            println!("{:?}", map);
        }
        Err(e) => {
            eprintln!("Failed to load the file: {}", e);
            std::process::exit(1);
        }
    }
    std::process::exit(0);
}
