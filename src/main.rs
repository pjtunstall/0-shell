mod commands;
mod helpers;

#[cfg(test)]
mod test_helpers;

use std::{
    collections::VecDeque,
    io::{self, Write},
    process,
};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use commands::{cd::cd, cp::cp, echo::echo, exit::exit, ls::ls, mkdir::mkdir, mv::mv, pwd::pwd};

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

fn red_println(text: &str) {
    println!("\x1b[31m{}\x1b[0m\x1b[1m", text);
}

fn main() {
    let _bold_text = TextStyle::new();
    let mut history = VecDeque::new();

    loop {
        let input = match get_input(&mut history) {
            Ok(input) if input.is_empty() => continue,
            Ok(input) => input,
            Err(_) => {
                process::exit(0);
            }
        };

        history.push_back(input.clone());

        let splitput: Vec<String>;
        match helpers::split(&input) {
            Ok(res) => {
                splitput = res;
            }
            Err(err) => {
                red_println(&format!("0-shell: {}", &err));
                continue;
            }
        }

        if splitput.is_empty() {
            red_println(&format!("0-shell: parse error near `\\n'"));
            continue;
        }

        let command = splitput[0].as_str();

        let result = match command {
            "cat" => commands::cat::cat(&splitput),
            "cd" => cd(&splitput),
            "cp" => cp(&splitput),
            "echo" => echo(&splitput),
            "exit" => exit(&splitput),
            "mkdir" => mkdir(&splitput),
            "mv" => mv(&splitput),
            "pwd" => pwd(&splitput),
            "ls" => ls(&splitput),
            _ => {
                red_println(&format!("0-shell: command not found: {}", command));
                continue;
            }
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

fn handle_error(command: &str, err: String) {
    red_println(&format!("{}: {}", command, err.to_lowercase()));
}

fn get_input(history: &mut VecDeque<String>) -> io::Result<String> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode()?;
    let mut input = String::new();
    let mut cursor = 0;

    let prompt = prompt()?;
    write!(stdout, "\r{}{}", prompt, termion::cursor::Show).unwrap();
    stdout.flush().unwrap();

    for key in stdin.keys() {
        match key.unwrap() {
            Key::Char('\n') => {
                write!(stdout, "\r\n").unwrap();
                stdout.flush().unwrap();
                break;
            }
            Key::Char(c) => {
                input.insert(cursor, c);
                cursor += 1;
            }
            Key::Backspace => {
                if cursor > 0 {
                    cursor -= 1;
                    input.remove(cursor);
                }
            }
            Key::Left => {
                cursor = cursor.saturating_sub(1); // Prevents underflow
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
            Key::Ctrl('c') | Key::Ctrl('d') => {
                // Eventually change 'c' to handle internal processes without exiting 0-shell
                write!(stdout, "\r\n").unwrap();
                stdout.suspend_raw_mode().unwrap(); // Ensure terminal is reset
                process::exit(0);
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

fn prompt() -> io::Result<String> {
    let cwd = std::env::current_dir()?.display().to_string();
    let prompt = format!("{} ▶ ", cwd);
    Ok(prompt)
}
