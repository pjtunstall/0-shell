use std::collections::VecDeque;

use zero_shell::{
    commands::{cat, cd, cp, echo, exit, ls, man, mkdir, mv, pwd, rm, touch},
    input,
};

struct TextStyle;

impl TextStyle {
    fn new() -> Self {
        print!("\x1b[1m"); // Be bold

        TextStyle
    }
}

impl Drop for TextStyle {
    fn drop(&mut self) {
        // Reset formatting to normal when the item is dropped,
        print!("\x1b[0m"); // i.e. when the program ends.
    }
}

fn main() {
    let _style = TextStyle::new();
    let mut history = VecDeque::new();
    history.push_back(String::new());

    loop {
        let input_string = match input::get_input(&mut history) {
            Ok(ok_input) => ok_input,
            Err(err) => {
                let text = format!("0-shell: failed to get input: {}", err);
                red_println(&text);
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
                red_println(&format!("0-shell: {}", &err));
                continue;
            }
        }

        if input_after_splitting.is_empty() {
            red_println(&format!("0-shell: parse error near `\\n'"));
            continue;
        }

        let command = &input_after_splitting[0];
        let result = match_command(&input_after_splitting);

        match result {
            Ok(ok) => {
                if !ok.is_empty() && command != "cat" {
                    print!("{}", &ok);
                }
            }
            Err(err) => handle_error(command, err),
        }
    }
}

fn match_command(input_after_splitting: &[String]) -> Result<String, String> {
    let command = input_after_splitting[0].as_str();
    match command {
        "cat" => cat::cat(&input_after_splitting),
        "cd" => cd::cd(&input_after_splitting),
        "cp" => cp::cp(&input_after_splitting),
        "echo" => echo::echo(&input_after_splitting),
        "exit" => exit::exit(&input_after_splitting),
        "ls" => ls::ls(&input_after_splitting),
        "mkdir" => mkdir::mkdir(&input_after_splitting),
        "man" => man::man(&input_after_splitting),
        "mv" => mv::mv(&input_after_splitting),
        "pwd" => pwd::pwd(&input_after_splitting),
        "rm" => rm::rm(&input_after_splitting),
        "touch" => touch::touch(&input_after_splitting),
        _ => Err(format!("command not found: {}", command)),
    }
}

fn handle_error(command: &str, err: String) {
    if err.starts_with("0-shell: ") {
        red_println(&format!("{}", err));
    } else {
        red_println(&format!("{}: {}", command, err));
    }
}

fn red_println(text: &str) {
    println!("\x1b[31m{}\x1b[0m\x1b[1m", text);
}
