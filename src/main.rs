mod hashtag;
mod parse_history;
mod setting;
mod ui;

use parse_history::get_command_hashmap;
use setting::command_line_setting;
use std::process::exit;
use ui::init_ui;

fn main() {
    command_line_setting();
    let command_hashmap = get_command_hashmap();
    init_ui(command_hashmap);
    exit(0);
}
