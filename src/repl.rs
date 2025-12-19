use std::collections::VecDeque;

use crate::{
    c::{self, SIGINT, SIGTSTP},
    commands::{
        self,
        jobs::{self, Job},
    },
    error, input,
};

struct TextStyle;

impl TextStyle {
    fn new() -> Self {
        print!("\x1b[1m");
        TextStyle
    }
}

impl Drop for TextStyle {
    fn drop(&mut self) {
        print!("\x1b[0m");
    }
}

pub fn repl() {
    let mut jobs: Vec<Job> = Vec::new();

    unsafe {
        c::signal(SIGINT, c::handle_forwarding);
        c::signal(SIGTSTP, c::handle_forwarding);
    }

    let _style = TextStyle::new();
    let mut history = VecDeque::new();
    history.push_back(String::new());

    loop {
        jobs::check_background_jobs(&mut jobs);

        let input_string = match input::get_input(&mut history) {
            Ok(ok_input) => ok_input,
            Err(err) => {
                let text = format!("0-shell: Failed to get input: {}", err);
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
            error::red_println(&format!("0-shell: Parse error near `\\n'"));
            continue;
        }

        commands::run_command(&input_after_splitting, &mut jobs);
    }
}
