pub fn handle_error(command: &str, err: String) {
    if err.starts_with("0-shell: ") {
        red_println(&format!("{}", err));
    } else {
        red_println(&format!("{}: {}", command, err));
    }
}

pub fn red_println(text: &str) {
    println!("\x1b[31m{}\x1b[0m\x1b[1m", text);
}
