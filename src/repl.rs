use std::collections::VecDeque;

use crate::{c::*, commands, error, input};

struct TextStyle;

impl TextStyle {
    fn new() -> Self {
        print!("\x1b[1m"); // Be bold!

        TextStyle
    }
}

impl Drop for TextStyle {
    fn drop(&mut self) {
        // Reset formatting to normal when the item is dropped,
        print!("\x1b[0m"); // i.e. when the program ends.
    }
}

pub fn repl() {
    unsafe {
        // Register the handler: tell the OS to call `handle_sigint` when a `SIGINT` signal is received.
        signal(SIGINT, handle_sigint);
    }

    let _style = TextStyle::new();
    let mut history = VecDeque::new();
    history.push_back(String::new());

    loop {
        let input_string = match input::get_input(&mut history) {
            Ok(ok_input) => ok_input,
            Err(err) => {
                let text = format!("0-shell: failed to get input: {}", err);
                error::red_println(&text);
                continue;
            }
        };

        if input_string.is_empty() {
            continue;
        };
        history.push_back(input_string.clone());

        let input_after_splitting: Vec<String>;
        match input::split::split(&input_string) {
            Ok(res) => {
                input_after_splitting = res;
            }
            Err(err) => {
                error::red_println(&format!("0-shell: {}", &err));
                continue;
            }
        }

        if input_after_splitting.is_empty() {
            error::red_println(&format!("0-shell: parse error near `\\n'"));
            continue;
        }

        commands::run_command(&input_after_splitting);
    }
}
