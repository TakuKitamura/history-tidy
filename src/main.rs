mod hashtag;
mod parse_history;
mod setting;
mod ui;
mod utils;

use parse_history::*;
use setting::command_line_setting;
use std::process::exit;
use ui::init_ui;

fn main() {
    command_line_setting();
    let history_vec: Vec<String> = match get_tidy_history() {
        Ok(history_vec) => history_vec,
        Err(e) => {
            println!("{}", e);
            vec![]
        }
    };
    if history_vec.len() == 0 {
        println!("No history found");
        exit(0);
    }
    let command_hashmap = get_command_hashmap(history_vec);
    init_ui(command_hashmap);
    exit(0);
}
