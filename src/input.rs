mod backtrack;
pub mod split;

use std::{
    collections::VecDeque,
    env, fs,
    io::{self, Stdout, Write},
    path::MAIN_SEPARATOR,
    process,
};

use lazy_static::lazy_static;
use termion::{
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
};

use crate::commands::{ls, rm};

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

pub fn get_input(history: &mut VecDeque<String>) -> io::Result<String> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode()?;
    let mut input = String::new();
    let mut cursor = 0;

    let prompt = prompt()?;
    write!(stdout, "\r{}{}", prompt, termion::cursor::Show).unwrap();
    stdout.flush().unwrap();

    for byte in stdin.keys() {
        match byte {
            Ok(key) => {
                match key {
                    Key::Char('¨') => {
                        // Necessary, together with the error check, to prevent panic on Option + `u`.
                        continue;
                    }
                    Key::Ctrl('c') | Key::Ctrl('d') => {
                        // Eventually change 'c' to handle internal processes without exiting 0-shell
                        write!(stdout, "\r\n").unwrap();
                        stdout.suspend_raw_mode().unwrap(); // Ensure terminal is reset
                        process::exit(0);
                    }
                    Key::Ctrl('u') => {
                        input.clear();
                        cursor = 0;
                    }
                    Key::Char('\n') => {
                        write!(stdout, "\r\n").unwrap();
                        stdout.flush().unwrap();
                        break;
                    }
                    Key::Char('\t') => {
                        if tab_and_should_continue(&mut input, &mut cursor, &mut stdout, &prompt) {
                            continue;
                        }
                    }
                    Key::Char(c) => {
                        debug_assert!(
                            cursor <= input.len(),
                            "Cursor should not be greater than length of input"
                        );
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
            Err(_) => {
                continue;
            }
        }
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

fn tab_and_should_continue(
    input: &mut String,
    cursor: &mut usize,
    stdout: &mut RawTerminal<Stdout>,
    prompt: &str,
) -> bool {
    let last_char = input.chars().last();
    let words = input.split_whitespace().collect::<Vec<&str>>();
    let waiting_for_cmd =
        words.is_empty() || (words.len() == 1 && last_char.map_or(false, |c| c != ' '));

    let data: &[String];
    let partial;
    let matches;

    if waiting_for_cmd {
        data = &COMMANDS;
        partial = if words.is_empty() {
            ""
        } else {
            words.last().unwrap()
        };
        matches = backtrack::find_matches(data, partial);
    } else {
        // Looking for an option?
        if words.len() > 1 && *words.last().unwrap() == "-" {
            let message;
            match words[0] {
                "rm" => message = rm::OPTIONS_USAGE,
                "ls" => message = ls::OPTIONS_USAGE,
                _ => message = "",
            }
            display_usage(stdout, message, prompt, input);
            write!(stdout, "\r{}{}", prompt, input).unwrap();
        }

        // Looking for a file or folder?
        let current_dir = env::current_dir().unwrap();
        let paths = fs::read_dir(&current_dir)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map_or(false, |name| !name.starts_with("."))
            })
            .map(|path| {
                let mut name = path.file_name().unwrap().to_string_lossy().to_string();
                if path.is_dir() {
                    name.push(MAIN_SEPARATOR);
                }
                name
            })
            .collect::<Vec<String>>();
        data = &paths;
        partial = if last_char.map_or(false, |c| c == ' ') {
            ""
        } else {
            words.last().unwrap()
        };
        matches = backtrack::find_matches(data, partial);
    }

    match matches.len() {
        0 => true, // No matches, continue
        1 => {
            // Handle single match - replace the partial with the complete match
            if words.len() > 0 {
                *input = words[..words.len().saturating_sub(1)].join(" ");
                if !input.is_empty() && !input.ends_with(' ') {
                    input.push(' ');
                }
            } else {
                input.clear();
            }

            input.push_str(&matches[0]);
            input.push(' ');
            *cursor = input.len();

            // Redraw the prompt with the updated input
            write!(stdout, "\r\x1b[K{}{}", prompt, input).unwrap();
            stdout.flush().unwrap();

            false // Don't continue, we've handled it
        }
        _ => {
            // Multiple matches, display them
            display_matches(stdout, matches, prompt, input);
            true // Continue to avoid overwriting with the prompt
        }
    }
}

fn display_matches(
    stdout: &mut RawTerminal<Stdout>,
    matches: Vec<String>,
    prompt: &str,
    input: &str,
) {
    if matches.is_empty() {
        // Shouldn't be called if matches is empty, but just in case ...
        return;
    }

    let (term_width, _) = termion::terminal_size().unwrap_or((80, 24));
    let term_width = term_width as usize;

    let max_len = matches.iter().map(|s| s.len()).max().unwrap_or(0);
    let col_width = max_len + 2; // Add spacing between columns

    let num_cols = (term_width / col_width).max(1);

    let num_items = matches.len();
    let num_rows = (num_items + num_cols - 1) / num_cols;

    write!(stdout, "\r\n").unwrap();

    for row in 0..num_rows {
        for col in 0..num_cols {
            let idx = row + (col * num_rows);
            if idx < num_items {
                let entry = &matches[idx];
                // Make sure we don't exceed terminal width
                if col * col_width + entry.len() < term_width {
                    write!(stdout, "{:<width$}", entry, width = col_width).unwrap();
                }
            }
        }
        if row < num_rows - 1 {
            write!(stdout, "\r\n").unwrap();
        }
    }

    // Move the cursor back up by the exact number of rows we displayed
    write!(stdout, "\r\x1b[{}A", num_rows).unwrap();

    // Redraw the prompt and input
    write!(stdout, "\r{}{}", prompt, input).unwrap();
    stdout.flush().unwrap();
}

fn display_usage(stdout: &mut RawTerminal<Stdout>, message: &str, prompt: &str, input: &str) {
    if message.is_empty() {
        return;
    }
    let lines = message.matches('\n').count();
    write!(stdout, "{}", message).unwrap();
    write!(stdout, "\r\x1b[{}A", lines).unwrap();
    write!(stdout, "\r{}{}", prompt, input).unwrap();
    stdout.flush().unwrap();
}
