use crate::hashtag::Hashtag;
use crate::hashtag::HashtagParser;
use dirs::home_dir;
use std::fs::read_to_string;
use std::io::Error;
use std::io::ErrorKind;

use linked_hash_map::LinkedHashMap;

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

pub fn get_command_hashmap(history_vec: Vec<String>) -> LinkedHashMap<String, Vec<String>> {
    let mut command_hashmap: LinkedHashMap<String, Vec<String>> = LinkedHashMap::new();
    let mut all: Vec<String> = vec![];
    for history in &history_vec {
        if history.len() == 0 {
            continue;
        }

        match all.iter().position(|v: &String| v == history) {
            Some(i) => {
                let latest_history: String = all.remove(i);
                all.push(latest_history);
            }
            None => all.push(history.to_owned()),
        }

        let hashtags: Vec<Hashtag> = HashtagParser::new(&history).collect::<Vec<Hashtag>>();

        for hashtag in hashtags {
            let hashtag: String = "#".to_owned() + &hashtag.text.to_owned().to_owned();
            if command_hashmap.contains_key(&hashtag) == false {
                let mut map_hashtag: Vec<String> = vec![];
                map_hashtag.push(history.to_owned());
                command_hashmap.insert(hashtag, map_hashtag);
            } else {
                let map_hashtag: &mut Vec<String> = command_hashmap.get_mut(&hashtag).unwrap();

                match map_hashtag
                    .into_iter()
                    .position(|item: &mut String| item == history)
                {
                    Some(i) => {
                        let latest_history: String = map_hashtag.remove(i);
                        map_hashtag.push(latest_history);
                    }
                    None => map_hashtag.push(history.to_owned()),
                }
            }
        }
    }

    command_hashmap.insert("ALL".to_owned(), all);
    return command_hashmap;
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
            ("#hoge", vec!["pwd #hoge", "cd ~ #hoge #fuga"]),
            ("#fuga", vec!["cd ~ #hoge #fuga"]),
            ("ALL", vec!["pwd #hoge", "cd ~ #hoge #fuga", "ls -a"]),
        ]
        .into_iter()
        .map(|(k1, v1)| {
            (
                k1.to_owned(),
                v1.into_iter()
                    .map(|v2| v2.to_owned())
                    .collect::<Vec<String>>(),
            )
        })
        .collect::<LinkedHashMap<String, Vec<String>>>();

        assert_eq!(command_hashmap, expected_command_hashmap);
    }
}
