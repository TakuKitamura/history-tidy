use clap::{crate_authors, crate_description, crate_version};
use clap::{App, Arg, SubCommand};
use dirs;

// get history vector with duplicates removed
fn get_tidy_history() -> Result<Vec<String>, std::io::Error> {
    match dirs::home_dir() {
        Some(mut history_file_path) => {
            history_file_path.push(".history-tidy");
            match std::fs::read_to_string(history_file_path) {
                Ok(history_file_content) => {
                    let mut history_vec: Vec<String> = history_file_content
                        .lines()
                        .map(|line| line.to_string().trim().to_string())
                        .collect();
                    history_vec.sort();
                    history_vec.dedup();
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
    let matches = App::new("history-tidy")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand(SubCommand::with_name("list").about("listing history with tags"))
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

    // no args or subcommand is 'init' case
    if let Some(matches) = matches.subcommand_matches("init") {
        let shell = matches.value_of("shell-type").unwrap();
        let init_shell_script = match shell {
            "bash" => include_str!("../bin/init.bash"),
            _ => unreachable!(),
        };
        println!("{}", init_shell_script);
       std::process::exit(0);
    }

    // no args or subcommand is 'list' case
    match get_tidy_history() {
        Ok(history_vec) => {
            for line in history_vec {
                println!("{}", line);
            }
        }
        Err(e) => {
            eprintln!("Failed to load the file: {}", e);
            std::process::exit(1);
        }
    }
    std::process::exit(0);
}
