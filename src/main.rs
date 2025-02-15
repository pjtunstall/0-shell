mod commands;
mod helpers;

use commands::{cd::cd, echo::echo, exit::exit, ls::ls, mkdir::mkdir, pwd::pwd};

use std::{
    io::{self, Write},
    process,
};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

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

    loop {
        // let prompt = prompt().unwrap_or_else(|err| {
        //     panic!("Failed to generate prompt: {}", err);
        // });
        // print!("{}", &prompt);
        // io::stdout().flush().expect("Failed to flush stdout"); // When you use functions like print!, println!, or other write operations to stdout, the output is typically buffered. This means that the data doesn't immediately go to the terminal or file but is stored temporarily in memory until it's flushed (or until the buffer is full). For example, if you use println!, it automatically appends a newline, which generally flushes the output, but in some cases (such as with print!), you need to explicitly flush the output to ensure it’s immediately written to the terminal.

        // Handle Ctrl + D (EOF) and other input errors
        let input = match get_input() {
            Ok(input) if input.is_empty() => continue, // Ignore empty input
            Ok(input) => input,
            Err(_) => {
                process::exit(0);
            }
        };

        let input = helpers::split(&input);
        if input.is_empty() {
            continue;
        }

        let command = input[0].as_str();

        let result = match command {
            "cd" => cd(&input),
            "echo" => echo(&input),
            "exit" => exit(&input),
            "mkdir" => mkdir(&input),
            "pwd" => pwd(&input),
            "ls" => ls(&input),
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

// fn prompt() -> io::Result<String> {
//     let cwd = helpers::get_current_dir()?;
//     let prompt = format!("{} ▶ ", cwd);
//     Ok(prompt)
// }

// fn get_input() -> io::Result<String> {
//     let mut input = String::new();
//     let bytes = io::stdin().lock().read_line(&mut input)?;

//     if bytes == 0 {
//         // EOF (Ctrl + D)
//         return Err(io::Error::new(io::ErrorKind::Other, "EOF reached"));
//     }

//     Ok(input.trim().to_string())
// }

fn get_input() -> io::Result<String> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode()?;
    let mut input = String::new();
    let mut cursor = 0;

    let prompt = prompt()?; // Get working directory prompt
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
            Key::Ctrl('c') | Key::Ctrl('d') => {
                // Eventually change 'c' to handle internal processes without exiting 0-shell
                write!(stdout, "\r\n").unwrap();
                stdout.suspend_raw_mode().unwrap(); // Ensure terminal is reset
                process::exit(0);
            }
            _ => {}
        }

        // Clear input area
        write!(stdout, "\r{}{}", prompt, termion::clear::AfterCursor).unwrap();
        write!(stdout, "{}", input).unwrap();

        // Move cursor **one space ahead** of the last typed character
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
