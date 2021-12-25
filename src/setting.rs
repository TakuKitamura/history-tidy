use colored::*;
use std::env;
use std::process::exit;

const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");
const PACKAGE_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

fn gen_help_string() -> String {
    return format!(
        "{package_name} {package_version}
{package_description}

{usage}:
    {package_name} [OPTIONS]

{options}:
    {help}      Prints help information
    {version}   Prints version information
    {init_bash}      Initialize bash history file",
        package_name = PACKAGE_NAME,
        package_version = PACKAGE_VERSION,
        package_description = PACKAGE_DESCRIPTION,
        usage = "USAGE".cyan().bold(),
        options = "OPTIONS".cyan().bold(),
        help = "-h, --help".green(),
        version = "-V, --version".green(),
        init_bash = "-init-bash".green()
    );
}

pub fn command_line_setting() {
    let args: Vec<String> = env::args().collect();

    if args.len() >= 2 {
        let option: &String = &args[1];
        if option == "-h" || option == "--help" {
            println!("{}", gen_help_string());
            exit(0);
        } else if option == "-V" || option == "--version" {
            println!("{} {}", PACKAGE_NAME, PACKAGE_VERSION);
            exit(0);
        } else if option == "-init-bash" {
            println!("{}", include_str!("../bin/init.bash"));
            exit(0);
        } else {
            eprintln!(
                "{}: Unknown argument '{}'\n",
                "error".red().bold(),
                option.yellow()
            );
            println!("{}", gen_help_string());
            exit(1);
        }
    }
}
