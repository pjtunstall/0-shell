mod backtrack;
pub mod split;

use std::{
    collections::VecDeque,
    env, fs,
    io::{self, Stdout, Write},
    process,
};

use lazy_static::lazy_static;
use termion::{
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
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

use crate::commands::ls::format;

pub fn get_input(history: &mut VecDeque<String>) -> io::Result<String> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode()?;
    let mut input = String::new();
    let mut cursor = 0;
    let mut num_spaces: usize = 0;

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
                        let mut is_cmd = true;
                        if num_spaces > 0 {
                            is_cmd = false;
                        }
                        let mut words = input.split_whitespace().collect::<Vec<_>>();
                        let last_word = words.pop().unwrap_or("");
                        let trimmed_input = words.join(" ");
                        let matches = find_command_or_file(last_word, is_cmd);
                        match matches.len() {
                            0 => continue,
                            1 => {
                                input = trimmed_input;
                                if let Some(c) = input.chars().last() {
                                    if c != ' ' {
                                        input.push(' ');
                                    }
                                }
                                input.push_str(matches[0].as_str());
                                input.push(' ');
                                cursor = input.len();
                                num_spaces += 1;
                            }
                            _ => {
                                display_matches(&mut stdout, matches, &prompt, &input);
                                continue; // To avoid overwriting with the prompt
                            }
                        }
                    }
                    Key::Char(c) => {
                        debug_assert!(
                            cursor <= input.len(),
                            "Cursor should not be greater than length of input"
                        );
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

fn find_command_or_file(last_word: &str, is_cmd: bool) -> Vec<String> {
    let entries;

    let data: &[String] = if is_cmd {
        &COMMANDS
    } else {
        entries = fs::read_dir(env::current_dir().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect::<Vec<String>>();

        &entries
    };

    backtrack::find_matches(data, last_word)
}

fn display_matches(
    stdout: &mut RawTerminal<Stdout>,
    matches: Vec<String>,
    prompt: &str,
    input: &str,
) {
    let matches = format::short_format_list(matches)
        .unwrap_or(String::new())
        .trim_end_matches('\n')
        .to_string();

    let lines = matches.matches('\n').count() + 1;

    write!(stdout, "\r\n{}", matches).unwrap();
    write!(stdout, "\x1b[{}A", lines).unwrap(); // Move up by the number of lines in the formatted output and thus back to the beginning of the prompt
    write!(stdout, "\r{}{}{}", prompt, input, termion::cursor::Show).unwrap();
    stdout.flush().unwrap();
}
