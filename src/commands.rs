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

use std::env;
use std::ffi::CString;
use std::ptr;
#[cfg(unix)]
use std::sync::atomic::Ordering;

use crate::{
    c::{self, *},
    commands::jobs::{Job, State},
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

        // Custom commands: fork and let the child exec this program (plus command name and other args).
        "cat" | "cp" | "ls" | "mkdir" | "man" | "mv" | "rm" | "sleep" | "touch" => {
            spawn_job(clean_args, jobs, is_background, true, current, previous)
        }

        // External commands: fork and let the child exec the external binary (plus other args).
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
    let clean_args = &args[2..];

    let result = match command {
        "cat" => cat::cat(clean_args),
        "cp" => cp::cp(clean_args),
        "ls" => ls::ls(clean_args),
        "mkdir" => mkdir::mkdir(clean_args),
        "man" => man::man(clean_args),
        "mv" => mv::mv(clean_args),
        "rm" => rm::rm(clean_args),
        "sleep" => sleep::sleep(clean_args),
        "touch" => touch::touch(clean_args),
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
    let pid = unsafe { c::fork() };

    if pid < 0 {
        return Err("Fork failed".to_string());
    }

    if pid == 0 {
        // In this branch, we're the CHILD.
        unsafe {
            let child_pid = c::getpid();
            c::setpgid(0, 0);

            if !is_background {
                c::tcsetpgrp(c::STDIN_FILENO, child_pid);
            }

            // Reset signal handlers to default behavior.
            c::signal(SIGINT, SIG_DFL);
            c::signal(SIGQUIT, SIG_DFL);
            c::signal(SIGTSTP, SIG_DFL);
            c::signal(SIGTTIN, SIG_DFL);
            c::signal(SIGTTOU, SIG_DFL);
            c::signal(SIGCHLD, SIG_DFL);

            // This is the vector (`argv` array) that we'll re-exec, i.e. the name of the program (along with any arguments) that will take over from the current process when we call `c::execvp` at the end of this (the child's) branch.
            let exec_args: Vec<String> = if is_worker {
                // The path of this program, followed by the command (name of custom-implemented command), followed by the internal-worker flag, then any other arguments.
                let self_path = env::current_exe()
                    .unwrap_or_else(|_| std::path::PathBuf::from("./0-shell"))
                    .to_string_lossy()
                    .into_owned();
                let mut v = vec![self_path, "--internal-worker".to_string()];
                v.extend_from_slice(args);
                v
            } else {
                // The command (name of external binary), followed by any other arguments.
                args.to_vec()
            };

            let c_strings: Vec<CString> = exec_args
                .into_iter()
                .map(|s| CString::new(s).unwrap())
                .collect();
            let mut ptrs: Vec<*const i8> = c_strings.iter().map(|s| s.as_ptr()).collect();
            ptrs.push(ptr::null());

            // exec vector args + PATH lookup: replace current process image with the target program, keeping the same PID.
            c::execvp(ptrs[0], ptrs.as_ptr());

            // Only runs if `execvp` fails.
            std::process::exit(1);
        }
    } else {
        // In this branch, we're the PARENT.
        unsafe {
            c::setpgid(pid, pid);

            if is_background {
                let id = jobs.len() + 1;
                let cmd_str = args.join(" ");
                jobs.push(Job::new(id, pid, cmd_str, State::Running));
                *previous = *current;
                *current = id;
                return Ok(format!("[{}] {}\n", id, pid));
            } else {
                c::tcsetpgrp(c::STDIN_FILENO, pid);
                c::CURRENT_CHILD_PID.store(pid, Ordering::SeqCst);

                let mut status = 0;
                c::waitpid(pid, &mut status, c::WUNTRACED);

                c::tcsetpgrp(c::STDIN_FILENO, c::getpgrp());
                c::CURRENT_CHILD_PID.store(0, Ordering::SeqCst);

                if c::w_if_stopped(status) {
                    let id = jobs.len() + 1;
                    let cmd_str = args.join(" ");
                    jobs.push(Job::new(id, pid, cmd_str.clone(), State::Stopped));
                    *previous = *current;
                    *current = id;
                    println!("\n[{}]+\tStopped\t\t{}", id, cmd_str);
                }
            }
        }
    }
    Ok(String::new())
}
