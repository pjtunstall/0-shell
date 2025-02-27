use std::{
    collections::VecDeque,
    io::{self, Write},
    process,
};

use lazy_static::lazy_static;
use termion::{event::Key, input::TermRead, raw::IntoRawMode};

use zero_shell::{
    backtrack,
    commands::{
        cat::cat,
        cd::cd,
        cp::cp,
        echo::echo,
        exit::exit,
        ls::{format, ls},
        mkdir::mkdir,
        mv::mv,
        pwd::pwd,
        rm::rm,
        touch::touch,
    },
    helpers,
};

lazy_static! {
    static ref COMMANDS: Vec<String> = vec![
        "cat".to_string(),
        "cd".to_string(),
        "cp".to_string(),
        "echo".to_string(),
        "exit".to_string(),
        "ls".to_string(),
        "mkdir".to_string(),
        "mv".to_string(),
        "pwd".to_string(),
        "rm".to_string(),
        "touch".to_string(),
    ];
}

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
        print!("\x1b[0m"); // i.e. when the program ends
    }
}

fn main() {
    let _style = TextStyle::new();
    let mut history = VecDeque::new();
    history.push_back(String::new());

    loop {
        let input_string = match get_input(&mut history) {
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
        match helpers::split(&input_string) {
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

        let result = match_command(&input_after_splitting);
        let command = if result.is_ok() {
            &input_after_splitting[0]
        } else {
            "0-shell"
        };

        match result {
            Ok(ok) => {
                if !ok.is_empty() {
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
        "cat" => cat(&input_after_splitting),
        "cd" => cd(&input_after_splitting),
        "cp" => cp(&input_after_splitting),
        "echo" => echo(&input_after_splitting),
        "exit" => exit(&input_after_splitting),
        "ls" => ls(&input_after_splitting),
        "mkdir" => mkdir(&input_after_splitting),
        "mv" => mv(&input_after_splitting),
        "pwd" => pwd(&input_after_splitting),
        "rm" => rm(&input_after_splitting),
        "touch" => touch(&input_after_splitting),
        _ => Err(format!("command not found: {}", command)),
    }
}

fn handle_error(command: &str, err: String) {
    red_println(&format!("{}: {}", command, err));
}

fn red_println(text: &str) {
    println!("\x1b[31m{}\x1b[0m\x1b[1m", text);
}

fn prompt() -> io::Result<String> {
    let cwd = std::env::current_dir()?.display().to_string();
    let prompt = format!("{} ▶ ", cwd);
    Ok(prompt)
}

fn get_input(history: &mut VecDeque<String>) -> io::Result<String> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode()?;
    let mut input = String::new();
    let mut cursor = 0;
    let mut num_spaces: usize = 0;

    let prompt = prompt()?;
    write!(stdout, "\r{}{}", prompt, termion::cursor::Show).unwrap();
    stdout.flush().unwrap();

    for key in stdin.keys() {
        match key.unwrap() {
            Key::Ctrl('c') | Key::Ctrl('d') => {
                // Eventually change 'c' to handle internal processes without exiting 0-shell
                write!(stdout, "\r\n").unwrap();
                stdout.suspend_raw_mode().unwrap(); // Ensure terminal is reset
                process::exit(0);
            }
            Key::Char('\n') => {
                write!(stdout, "\r\n").unwrap();
                stdout.flush().unwrap();
                break;
            }
            Key::Char('\t') => {
                if num_spaces > 0 {
                    continue;
                }
                let matches = tab(&input);
                if matches.len() == 1 {
                    input.clear();
                    input.push_str(matches[0].as_str());
                    input.push(' ');
                    cursor = input.len();
                    num_spaces += 1;
                } else {
                    let matches = format::short_format_list(matches).unwrap_or(String::new());

                    let lines = matches.chars().filter(|c| *c == '\n').count() + 1;

                    write!(stdout, "\r\n{}", matches).unwrap();
                    write!(stdout, "\x1b[{}A", lines).unwrap(); // Move up by matches' lines, back to the beginning of the prompt
                    write!(stdout, "\r{}{}{}", prompt, input, termion::cursor::Show).unwrap();
                    stdout.flush().unwrap();

                    continue; // To avoid overwriting with the prompt
                }
            }
            Key::Char(c) => {
                input.insert(cursor, c);
                cursor += 1;
                if c == ' ' {
                    num_spaces += 1;
                }
            }
            Key::Backspace => {
                if cursor > 0 {
                    cursor -= 1;
                    if let Some(last) = input.chars().last() {
                        if last == ' ' {
                            num_spaces -= 1;
                        }
                    }
                    input.remove(cursor);
                }
            }
            Key::Left => {
                cursor = cursor.saturating_sub(1);
            }
            Key::Right => {
                if cursor < input.len() {
                    cursor += 1;
                }
            }
            Key::Up => {
                if !history.is_empty() {
                    history.rotate_right(1);
                    if let Some(item) = history.front() {
                        input = item.clone();
                        cursor = input.len();
                    }
                }
            }
            Key::Down => {
                if !history.is_empty() {
                    history.rotate_left(1);
                    if let Some(item) = history.front() {
                        input = item.clone();
                        cursor = input.len();
                    } else {
                        input.clear();
                        cursor = 0;
                    }
                }
            }
            _ => {}
        }

        write!(stdout, "\r{}{}", prompt, termion::clear::AfterCursor).unwrap();
        write!(stdout, "{}", input).unwrap();

        let move_left_by = input.len().saturating_sub(cursor);
        if move_left_by > 0 {
            write!(stdout, "{}", termion::cursor::Left(move_left_by as u16)).unwrap();
        }

        stdout.flush().unwrap();
    }

    // Restore terminal mode before returning
    stdout.suspend_raw_mode().unwrap();

    Ok(input)
}

fn tab(input: &str) -> Vec<String> {
    backtrack::find_matches(&COMMANDS, input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab() {
        let mut expected;

        expected = Vec::new();
        assert_eq!(
            tab("x"),
            expected,
            "`tab` should return an empty vector when there are no matches"
        );

        expected = vec!["cat".to_string(), "cd".to_string(), "cp".to_string()];
        assert_eq!(
            tab("c"),
            expected,
            "`tab(\"c\")` should find all three commands beginning with 'c'"
        );

        expected = vec!["mkdir".to_string()];
        assert_eq!(
            tab("mk"),
            expected,
            "`tab(\"mk\")` should return a vector containing just \"mkdir\""
        );
    }
}
