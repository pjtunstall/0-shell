mod backtrack;
pub mod split;

use std::{
    collections::VecDeque,
    env, fs,
    io::{self, Stdout, Write},
    path::MAIN_SEPARATOR,
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
        "bg".to_string(),
        "cat".to_string(),
        "cd".to_string(),
        "cp".to_string(),
        "echo".to_string(),
        "exit".to_string(),
        "fg".to_string(),
        "jobs".to_string(),
        "kill".to_string(),
        "ls".to_string(),
        "mkdir".to_string(),
        "man".to_string(),
        "mv".to_string(),
        "pwd".to_string(),
        "rm".to_string(),
        "sleep".to_string(),
        "touch".to_string(),
    ];
}

pub fn get_input(history: &mut VecDeque<String>) -> io::Result<Option<String>> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode()?;
    let mut input = String::new();
    let mut cursor = 0;

    let prompt = prompt()?;
    write!(stdout, "\r{}{}", prompt, termion::cursor::Show).expect("failed to write to `stdout`");
    stdout.flush().expect("failed to flush `stdout`");

    for byte in stdin.keys() {
        match byte {
            Ok(key) => {
                match key {
                    Key::Ctrl('d') => {
                        if input.is_empty() {
                            write!(stdout, "\r\n").expect("failed to write to `stdout`");
                            stdout.suspend_raw_mode().expect("failed to reset terminal");
                            return Ok(None);
                        }
                        continue;
                    }
                    Key::Char('¨') => {
                        continue;
                    }
                    Key::Ctrl('u') => {
                        input.clear();
                        cursor = 0;
                    }
                    Key::Char('\n') | Key::Ctrl('c') => {
                        write!(stdout, "\r\n").expect("failed to write to `stdout`");
                        stdout.flush().expect("failed to flush `stdout`");
                        break;
                    }
                    Key::Char('\t') => {
                        if tab_and_should_continue(&mut input, &mut cursor, &mut stdout, &prompt) {
                            continue;
                        }
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

                write!(stdout, "\r{}{}", prompt, termion::clear::AfterCursor)
                    .expect("failed to write to `stdout`");
                write!(stdout, "{}", input).expect("failed to write to `stdout`");

                let move_left_by = input.len().saturating_sub(cursor);
                if move_left_by > 0 {
                    write!(stdout, "{}", termion::cursor::Left(move_left_by as u16))
                        .expect("failed to write to `stdout`");
                }

                stdout.flush().expect("failed to flush `stdout`");
            }
            Err(_) => {
                continue;
            }
        }
    }

    stdout
        .suspend_raw_mode()
        .expect("failed to suspend raw mode");

    Ok(Some(input))
}

fn prompt() -> io::Result<String> {
    let cwd = std::env::current_dir()?.display().to_string();
    // Print cwd in blue; reset to default color afterwards.
    let prompt = format!("\x1b[34m{}\x1b[39m ▶ ", cwd);
    Ok(prompt)
}

// Return true to continue. This will bypass the usual prompt printing, which would overwrite any possibilities or options message. Only return false when one exact match is found.
fn tab_and_should_continue(
    input: &mut String,
    cursor: &mut usize,
    stdout: &mut RawTerminal<Stdout>,
    prompt: &str,
) -> bool {
    let last_char = input.chars().last();
    let current_input = input.clone();
    let words = current_input.split_whitespace().collect::<Vec<&str>>();

    if words.len() > 1 && *words.last().expect("expected a last word") == "-" {
        check_options(stdout, prompt, input, words[0]);
        return true; // On returning, continue.
    }

    let waiting_for_cmd =
        words.is_empty() || (words.len() == 1 && last_char.map_or(false, |c| c != ' '));

    let matches = if waiting_for_cmd {
        check_cmds(&words)
    } else {
        match check_paths(last_char, &words) {
            Some(v) => v,
            None => return true,
        }
    };

    match matches.len() {
        0 => true,
        1 => {
            display_match(stdout, cursor, prompt, input, &words, matches);
            false
        }
        _ => {
            display_possibilities(stdout, prompt, input, matches);
            true
        }
    }
}

fn check_cmds(words: &Vec<&str>) -> Vec<String> {
    let data = &COMMANDS;
    let partial = if words.is_empty() {
        ""
    } else {
        words.last().expect("expected there to be a last word")
    };
    backtrack::find_matches(data, partial)
}

fn check_options(stdout: &mut RawTerminal<Stdout>, prompt: &str, input: &str, cmd: &str) {
    let message;
    match cmd {
        "rm" => message = rm::OPTIONS_USAGE,
        "ls" => message = ls::OPTIONS_USAGE,
        _ => message = "",
    }
    display_usage(stdout, prompt, input, message);
    write!(stdout, "\r{}{}", prompt, input).expect("failed to write to `stdout`");
}

fn check_paths(last_char: Option<char>, words: &Vec<&str>) -> Option<Vec<String>> {
    let current_dir = match env::current_dir() {
        Ok(dir) => dir,
        Err(_) => {
            return None;
        }
    };

    let paths = fs::read_dir(&current_dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .map_or(false, |name| !name.starts_with("."))
                })
                .map(|path| {
                    path.file_name()
                        .map(|name| {
                            let mut name = name.to_string_lossy().to_string();
                            if path.is_dir() {
                                name.push(MAIN_SEPARATOR);
                            }
                            name
                        })
                        .unwrap_or_default()
                })
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();

    let data = &paths;
    let partial = if last_char.map_or(false, |c| c == ' ') {
        ""
    } else {
        words.last().expect("expected there to be a last word")
    };

    Some(backtrack::find_matches(data, partial))
}

fn display_match(
    stdout: &mut RawTerminal<Stdout>,
    cursor: &mut usize,
    prompt: &str,
    input: &mut String,
    words: &Vec<&str>,
    matches: Vec<String>,
) {
    if words.len() > 0 {
        // Replace partial with complete match.
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

    write!(stdout, "\r\x1b[K{}{}", prompt, input).expect("failed to write to `stdout`"); // Move the cursor back up.
    stdout.flush().expect("failed to flush `stdout`");
}

fn display_possibilities(
    stdout: &mut RawTerminal<Stdout>,
    prompt: &str,
    input: &str,
    matches: Vec<String>,
) {
    if matches.is_empty() {
        return;
    }

    let (term_width, _) = termion::terminal_size().unwrap_or((80, 24));
    let term_width = term_width as usize;

    let max_len = matches.iter().map(|s| s.len()).max().unwrap_or(0);
    let col_width = max_len + 2;

    let num_cols = (term_width / col_width).max(1);

    let num_items = matches.len();
    let num_rows = (num_items + num_cols - 1) / num_cols;

    write!(stdout, "\r\n").expect("failed to write to `stdout`");

    for row in 0..num_rows {
        for col in 0..num_cols {
            let idx = row + (col * num_rows);
            if idx < num_items {
                let entry = &matches[idx];
                if col * col_width + entry.len() < term_width {
                    write!(stdout, "{:<width$}", entry, width = col_width)
                        .expect("failed to write to `stdout`");
                }
            }
        }
        if row < num_rows - 1 {
            write!(stdout, "\r\n").expect("failed to write to `stdout`");
        }
    }

    write!(stdout, "\r\x1b[{}A", num_rows).expect("failed to write to `stdout`"); // Move the cursor back up.
    write!(stdout, "\r{}{}", prompt, input).expect("failed to write to `stdout`");
    stdout.flush().expect("failed to flush `stdout`");
}

fn display_usage(stdout: &mut RawTerminal<Stdout>, prompt: &str, input: &str, message: &str) {
    if message.is_empty() {
        return;
    }
    let num_rows = message.matches('\n').count();
    write!(stdout, "{}", message).expect("failed to write to `stdout`");
    write!(stdout, "\r\x1b[{}A", num_rows).expect("failed to write to `stdout`"); // Move the cursor back up.
    write!(stdout, "\r{}{}", prompt, input).expect("failed to write to `stdout`");
    stdout.flush().expect("failed to flush `stdout`");
}
