mod backtrack;
pub mod split;

use std::{
    collections::VecDeque,
    env, fs,
    io::{self, Stdout, Write},
    path::MAIN_SEPARATOR,
};

use dirs;
use lazy_static::lazy_static;
use termion::{
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
};
use whoami;

use crate::{
    ansi::{CLEAR_LINE, FOLDER_COLOR, RESET_FG, USER_COLOR},
    commands::{ls, rm},
};

lazy_static! {
    static ref COMMANDS: Vec<String> = vec![
        String::from("bg"),
        String::from("cat"),
        String::from("cd"),
        String::from("cp"),
        String::from("echo"),
        String::from("exit"),
        String::from("fg"),
        String::from("jobs"),
        String::from("kill"),
        String::from("ls"),
        String::from("mkdir"),
        String::from("man"),
        String::from("mv"),
        String::from("pwd"),
        String::from("rm"),
        String::from("sleep"),
        String::from("touch"),
    ];
}

pub fn get_input(history: &mut VecDeque<String>) -> io::Result<Option<String>> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode()?;

    unsafe {
        let mut ios = std::mem::zeroed::<libc::termios>();
        if libc::tcgetattr(libc::STDOUT_FILENO, &mut ios) == 0 {
            ios.c_oflag |= libc::OPOST;
            ios.c_oflag |= libc::ONLCR;
            libc::tcsetattr(libc::STDOUT_FILENO, libc::TCSANOW, &ios);
        }
    }

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
                            // Show ^D on its own line before any following output.
                            write!(stdout, "\r\n^D\r\n").expect("failed to write to `stdout`");
                            stdout.suspend_raw_mode().expect("failed to reset terminal");
                            return Ok(None);
                        }
                        continue;
                    }
                    Key::Ctrl('u') => {
                        input.clear();
                        cursor = 0;
                    }
                    Key::Char('\n') => {
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
                        cursor += c.len_utf8();
                    }
                    Key::Backspace => {
                        if cursor > 0 {
                            if let Some(c) = input[..cursor].chars().next_back() {
                                cursor -= c.len_utf8();
                                input.remove(cursor);
                            }
                        }
                    }
                    Key::Left => {
                        if cursor > 0 {
                            if let Some(c) = input[..cursor].chars().next_back() {
                                cursor -= c.len_utf8();
                            }
                        }
                    }
                    Key::Right => {
                        if cursor < input.len() {
                            if let Some(c) = input[cursor..].chars().next() {
                                cursor += c.len_utf8();
                            }
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

                let chars_remaining = input[cursor..].chars().count();
                if chars_remaining > 0 {
                    write!(stdout, "{}", termion::cursor::Left(chars_remaining as u16))
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
    let mut cwd = std::env::current_dir()?.display().to_string();
    let username = whoami::username();
    let hostname = whoami::devicename();

    if let Some(home_dir) = dirs::home_dir() {
        let home_str = home_dir.display().to_string();
        if cwd.starts_with(&home_str) {
            cwd = cwd.replacen(&home_str, "~", 1);
        }
    }

    let prompt =
        format!("{USER_COLOR}{username}@{hostname}{RESET_FG}:{FOLDER_COLOR}{cwd}{RESET_FG}$ ");
    Ok(prompt)
}

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
        return true;
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

    write!(stdout, "\r{CLEAR_LINE}{prompt}{input}").expect("failed to write to `stdout`");
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

    write!(stdout, "\r{}", crate::ansi::cursor_up(num_rows)).expect("failed to write to `stdout`");
    write!(stdout, "\r{}{}", prompt, input).expect("failed to write to `stdout`");
    stdout.flush().expect("failed to flush `stdout`");
}

fn display_usage(stdout: &mut RawTerminal<Stdout>, prompt: &str, input: &str, message: &str) {
    if message.is_empty() {
        return;
    }
    let num_rows = message.matches('\n').count();
    write!(stdout, "{}", message).expect("failed to write to `stdout`");
    write!(stdout, "\r{}", crate::ansi::cursor_up(num_rows)).expect("failed to write to `stdout`");
    write!(stdout, "\r{}{}", prompt, input).expect("failed to write to `stdout`");
    stdout.flush().expect("failed to flush `stdout`");
}
