use crate::ansi::{BOLD, RED, RESET};

pub fn handle_error(command: &str, err: String) {
    if err.starts_with("0-shell: ") {
        red_println(&format!("{}", err));
    } else {
        red_println(&format!("{}: {}", command, err));
    }
}

pub fn red_println(text: &str) {
    println!(
        "{RED}{text}{RESET}{BOLD}",
        text = text
    );
}
