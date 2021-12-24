use clap::{crate_authors, crate_description, crate_version};
use clap::{App, Arg, ArgMatches, SubCommand};
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
}
