// mod hashtag;

use clap::{crate_authors, crate_description, crate_version};
use clap::{App, Arg, ArgMatches, SubCommand};
use dirs::home_dir;
use std::fs::read_to_string;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;

pub fn command_line_setting() {
    let matches: ArgMatches = App::new("history-tidy")
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
        exit(0);
    }

    if let Some(_) = matches.subcommand_matches("load") {
        let script_path: PathBuf = match home_dir() {
            Some(mut history_file_path) => {
                history_file_path.push(".history-tidy");
                history_file_path.push("script");
                history_file_path
            }
            None => {
                return;
            }
        };
        let script_content: String = match read_to_string(&script_path) {
            Ok(script_content) => script_content,
            Err(e) => {
                println!("{}", e);
                return;
            }
        };

        let mut script_file: File = match OpenOptions::new()
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
        exit(0);
    }
}
