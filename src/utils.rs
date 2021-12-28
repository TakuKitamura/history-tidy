use colored::*;

pub fn error_exit(message: &str, err: std::io::Error, exit_code: i32) {
    eprintln!("{}: {} [{}]", "error".red(), message, err);
    std::process::exit(exit_code);
}
