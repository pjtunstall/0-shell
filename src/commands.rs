pub mod bg;
pub mod cat;
pub mod cd;
pub mod cp;
pub mod echo;
pub mod exit;
pub mod fg;
pub mod jobs;
pub mod kill;
pub mod ls;
pub mod man;
pub mod mkdir;
pub mod mv;
pub mod pwd;
pub mod rm;
pub mod sleep;
pub mod touch;

use std::{fs::OpenOptions, os::fd::AsRawFd};

use crate::{commands::jobs::Job, error, fork::spawn_job};

pub fn run_command(
    args: &[String],
    jobs: &mut Vec<Job>,
    current: &mut usize,
    previous: &mut usize,
    exit_attempted: &mut bool,
) -> (Option<i32>, Result<String, String>) {
    if args.is_empty() {
        return (None, Ok(String::new()));
    }

    let (clean_args, is_background_launch) = if let Some(last) = args.last() {
        if last == "&" {
            (&args[..args.len() - 1], true)
        } else {
            (args, false)
        }
    } else {
        (args, false)
    };

    if clean_args.is_empty() {
        return (None, Ok(String::new()));
    }

    let command = clean_args[0].as_str();

    if command != "exit" {
        *exit_attempted = false;
    }

    let mut exit_code: Option<i32> = None;

    let result = match command {
        // Basic commands: internal.
        "exit" => match exit::exit(clean_args, jobs, exit_attempted) {
            Ok(code_str) => {
                exit_code = Some(code_str.parse().unwrap_or(0));
                Ok(String::new())
            }
            Err(e) => Err(e),
        },
        "cd" => cd::cd(clean_args),
        "echo" => echo::echo(clean_args),
        "pwd" => pwd::pwd(clean_args),

        // Job control: internal.
        "bg" => bg::bg(clean_args, jobs, current, previous),
        "fg" => fg::fg(clean_args, jobs, current, previous),
        "jobs" => jobs::jobs(clean_args, jobs, current, previous),
        "kill" => kill::kill(clean_args, jobs, current, previous),

        // Custom commands: fork and let the child exec this program (plus
        // command name and other args).
        "cat" | "cp" | "ls" | "mkdir" | "man" | "mv" | "rm" | "sleep" | "touch" => spawn_job(
            clean_args,
            jobs,
            is_background_launch,
            true,
            current,
            previous,
        ),

        // External commands: fork and let the child exec the external binary
        // (plus other args).
        _ => spawn_job(
            clean_args,
            jobs,
            is_background_launch,
            false,
            current,
            previous,
        ),
    };

    (exit_code, result)
}

pub fn run_command_as_worker(args: &[String]) {
    if args.len() < 3 {
        error::red_println("0-shell: Internal worker error: missing command argument");
        return;
    }

    let command = args[2].as_str();
    let clean_args = match apply_fd_redirections(&args[2..]) {
        Ok(filtered) => filtered,
        Err(err) => {
            error::handle_error(command, err);
            return;
        }
    };

    let result = match command {
        "cat" => cat::cat(&clean_args),
        "cp" => cp::cp(&clean_args),
        "ls" => ls::ls(&clean_args),
        "mkdir" => mkdir::mkdir(&clean_args),
        "man" => man::man(&clean_args),
        "mv" => mv::mv(&clean_args),
        "rm" => rm::rm(&clean_args),
        "sleep" => sleep::sleep(&clean_args),
        "touch" => touch::touch(&clean_args),
        _ => Err(format!("Command not found: {}", command)),
    };

    match result {
        Ok(ok) => {
            if !ok.is_empty() {
                print!("{}", &ok);
            }
        }
        Err(err) => error::handle_error(command, err),
    }
}

// Handle redirections with an explicit file descriptor (e.g. `2>&1`).
// Returns the argv with those redirection tokens removed after applying them.
fn apply_fd_redirections(args: &[String]) -> Result<Vec<String>, String> {
    let mut filtered = Vec::with_capacity(args.len());
    let mut i = 0;

    while i < args.len() {
        if i + 2 < args.len() && i > 0 {
            if let Ok(fd) = args[i].parse::<i32>() {
                let op = args[i + 1].as_str();
                if op == ">" || op == ">>" {
                    let target = &args[i + 2];
                    apply_single_fd_redirection(fd, op, target)?;
                    i += 3;
                    continue;
                }
            }
        }

        filtered.push(args[i].clone());
        i += 1;
    }

    Ok(filtered)
}

fn apply_single_fd_redirection(fd: i32, op: &str, target: &str) -> Result<(), String> {
    if fd < 0 {
        return Err(String::from("Invalid file descriptor"));
    }

    if let Some(dup_target) = target.strip_prefix('&') {
        let target_fd = dup_target
            .parse::<i32>()
            .map_err(|_| format!("Bad file descriptor: {target}"))?;

        let res = unsafe { libc::dup2(target_fd, fd) };
        if res == -1 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        return Ok(());
    }

    let mut options = OpenOptions::new();
    options.write(true).create(true);

    if op == ">>" {
        options.append(true);
    } else {
        options.truncate(true);
    }

    let file = options
        .open(target)
        .map_err(|e| format!("Failed to open redirect target `{target}`: {e}"))?;

    let res = unsafe { libc::dup2(file.as_raw_fd(), fd) };
    if res == -1 {
        return Err(std::io::Error::last_os_error().to_string());
    }

    Ok(())
}
