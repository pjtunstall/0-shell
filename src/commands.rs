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

use std::{env, ffi::CString, fs::OpenOptions, io, os::fd::AsRawFd, ptr, sync::atomic::Ordering};

use crate::{
    commands::{
        fg::CURRENT_CHILD_PID,
        jobs::{Job, State},
    },
    error,
};

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

    let (clean_args, is_background) = if let Some(last) = args.last() {
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
        "cat" | "cp" | "ls" | "mkdir" | "man" | "mv" | "rm" | "sleep" | "touch" => {
            spawn_job(clean_args, jobs, is_background, true, current, previous)
        }

        // External commands: fork and let the child exec the external binary
        // (plus other args).
        _ => spawn_job(clean_args, jobs, is_background, false, current, previous),
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

fn spawn_job(
    args: &[String],
    jobs: &mut Vec<Job>,
    is_background: bool,
    is_worker: bool,
    current: &mut usize,
    previous: &mut usize,
) -> Result<String, String> {
    // Build argv up front so that the child doesn't have to allocate after the
    // fork.
    let exec_args: Vec<String> = if is_worker {
        let self_path = env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from("./0_shell"))
            .to_string_lossy()
            .into_owned();
        let mut v = vec![self_path, String::from("--internal-worker")];
        v.extend_from_slice(args);
        v
    } else {
        args.to_vec()
    };

    let c_strings: Vec<CString> = exec_args
        .into_iter()
        .map(|s| {
            CString::new(s).unwrap_or_else(|_| {
                eprintln!("0-shell: argument contains interior NUL byte");
                std::process::exit(1);
            })
        })
        .collect();
    let mut ptrs: Vec<*const i8> = c_strings.iter().map(|s| s.as_ptr()).collect();
    ptrs.push(ptr::null());

    let pid = unsafe { libc::fork() };

    match pid {
        -1 => Err(String::from("Fork failed")),
        0 => {
            run_child(ptrs, is_background, c_strings);
            unreachable!();
        }
        child_pid => run_parent(args, jobs, is_background, current, previous, child_pid),
    }
}

fn run_child(ptrs: Vec<*const i8>, is_background: bool, c_strings: Vec<CString>) {
    unsafe {
        // Child learns its own PID.
        let child_pid = libc::getpid();

        // Put child in its own process group (pgid = pid).
        libc::setpgid(0, 0);

        if !is_background {
            // Take control of the TTY for foreground jobs.
            libc::tcsetpgrp(libc::STDIN_FILENO, child_pid);
        }

        // Restore default signal handlers.
        libc::signal(libc::SIGINT, libc::SIG_DFL); // Ctrl+C
        libc::signal(libc::SIGTSTP, libc::SIG_DFL); // Ctrl+Z
        libc::signal(libc::SIGQUIT, libc::SIG_DFL); // Ctrl+\
        // Default: stop if background job reads the TTY.
        libc::signal(libc::SIGTTIN, libc::SIG_DFL);
        // Default: stop if background job writes the TTY.
        libc::signal(libc::SIGTTOU, libc::SIG_DFL);

        // Exec vector args + PATH lookup: that is, replace the current
        // process image with the target program, keeping the same PID.
        libc::execvp(ptrs[0], ptrs.as_ptr());

        // Only runs if `execvp` fails.
        error::print_exec_failure(c_strings[0].as_bytes());
        std::process::exit(1);
    }
}

fn run_parent(
    args: &[String],
    jobs: &mut Vec<Job>,
    is_background: bool,
    current: &mut usize,
    previous: &mut usize,
    child_pid: i32,
) -> Result<String, String> {
    unsafe {
        // Make the child the leader of a new process group.
        libc::setpgid(child_pid, child_pid);

        match is_background {
            true => {
                let id = jobs.len() + 1;
                let cmd_str = display_command(args);
                jobs.push(Job::new(id, child_pid, cmd_str, State::Running));
                *previous = *current;
                *current = id;
                return Ok(format!("[{}] {}\n", id, child_pid));
            }
            false => {
                // Hand terminal control to the foreground child.
                // It will now be be the recipient of any SIGINT or SIGTSTP.
                libc::tcsetpgrp(libc::STDIN_FILENO, child_pid);
                CURRENT_CHILD_PID.store(child_pid, Ordering::SeqCst);

                let status = wait_for_foreground_child(child_pid)?;

                // If the child was terminated by SIGINT, the cursor to the next
                // line so that the prompt doesn't overwrite the `^C`.
                if libc::WIFSIGNALED(status) && libc::WTERMSIG(status) == libc::SIGINT {
                    println!();
                }

                // Reclaim the terminal for the shell.
                libc::tcsetpgrp(libc::STDIN_FILENO, libc::getpgrp());
                CURRENT_CHILD_PID.store(0, Ordering::SeqCst);

                // The child was stopped: keep it in the jobs table.
                if libc::WIFSTOPPED(status) {
                    let id = jobs.len() + 1;
                    let cmd_str = display_command(args);
                    jobs.push(Job::new(id, child_pid, cmd_str.clone(), State::Stopped));
                    *previous = *current;
                    *current = id;
                    println!("\n[{}]+\tStopped\t\t{}", id, cmd_str);
                }
            }
        }
    }
    Ok(String::new())
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

// Blocks the shell process until the specific foreground child `child_pid`
// changes state. This handles both termination (exit) and suspension (Ctrl+Z).
fn wait_for_foreground_child(child_pid: i32) -> Result<i32, String> {
    let mut status = 0;

    loop {
        // `waitpid` normally only returns when a child terminates.
        // `WUNTRACED` tells it to also return if the child is stopped by a signal
        // (e.g., if the user hits Ctrl+Z). This is essential for Job Control.
        let res = unsafe { libc::waitpid(child_pid, &mut status, libc::WUNTRACED) };

        // Success: the specific child we were waiting for has changed state.
        if res == child_pid {
            return Ok(status);
        }

        // Failure or interruption
        if res == -1 {
            let err = io::Error::last_os_error();

            // EINTR means the `waitpid` system call itself was interrupted by a
            // signal caught by the shell (e.g., SIGWINCH or a SIGCHLD from a
            // distinct background process). This is temporary; we simply retry.
            if err.raw_os_error() == Some(libc::EINTR) {
                continue;
            }

            // Real failure (e.g., the child does not exist).
            return Err(format!("waitpid failed: {}", err));
        }

        // Unexpected: waitpid returned a PID we didn't ask for.
        // (This should technically be impossible when passing a positive PID).
        return Err(format!("waitpid returned unexpected pid: {}", res));
    }
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

fn display_command(args: &[String]) -> String {
    let mut display_parts: Vec<String> = Vec::with_capacity(args.len());
    let mut i = 0;

    while i < args.len() {
        if i + 2 < args.len() {
            if args[i].parse::<i32>().is_ok() {
                let op = args[i + 1].as_str();
                if (op == ">" || op == ">>") && !args[i + 2].is_empty() {
                    display_parts.push(format!("{}{}{}", args[i], op, args[i + 2]));
                    i += 3;
                    continue;
                }
            }
        }

        display_parts.push(args[i].clone());
        i += 1;
    }

    display_parts.join(" ")
}
