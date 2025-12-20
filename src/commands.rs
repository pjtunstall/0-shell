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
use std::io;
use std::ptr;
use std::sync::atomic::Ordering;

use crate::{
    c::CURRENT_CHILD_PID,
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
    // Build argv up front so the child avoids allocation after fork.
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

    if pid < 0 {
        return Err(String::from("Fork failed"));
    }

    if pid == 0 {
        // In this branch, we're the CHILD.
        unsafe {
            let child_pid = libc::getpid();
            libc::setpgid(0, 0);

            if !is_background {
                libc::tcsetpgrp(libc::STDIN_FILENO, child_pid);
            }

            // Reset signal handlers to default behavior.
            libc::signal(libc::SIGINT, libc::SIG_DFL);
            libc::signal(libc::SIGQUIT, libc::SIG_DFL);
            libc::signal(libc::SIGTSTP, libc::SIG_DFL);
            libc::signal(libc::SIGTTIN, libc::SIG_DFL);
            libc::signal(libc::SIGTTOU, libc::SIG_DFL);
            libc::signal(libc::SIGCHLD, libc::SIG_DFL);

            // exec vector args + PATH lookup: replace current process image with the target program, keeping the same PID.
            libc::execvp(ptrs[0], ptrs.as_ptr());

            // Only runs if `execvp` fails.
            error::print_exec_failure(c_strings[0].as_bytes());
            std::process::exit(1);
        }
    } else {
        // In this branch, we're the PARENT.
        unsafe {
            libc::setpgid(pid, pid);

            match is_background {
                true => {
                    let id = jobs.len() + 1;
                    let cmd_str = args.join(" ");
                    jobs.push(Job::new(id, pid, cmd_str, State::Running));
                    *previous = *current;
                    *current = id;
                    return Ok(format!("[{}] {}\n", id, pid));
                }
                false => {
                    libc::tcsetpgrp(libc::STDIN_FILENO, pid);
                    CURRENT_CHILD_PID.store(pid, Ordering::SeqCst);

                    let mut status = 0;
                    loop {
                        let res = libc::waitpid(pid, &mut status, libc::WUNTRACED);
                        if res == pid {
                            break;
                        }
                        if res == -1 {
                            let err = io::Error::last_os_error();
                            if err.raw_os_error() == Some(libc::EINTR) {
                                continue;
                            }
                            return Err(format!("waitpid failed: {}", err));
                        }
                        return Err(format!("waitpid returned unexpected pid: {}", res));
                    }

                    if libc::WIFSIGNALED(status) && libc::WTERMSIG(status) == libc::SIGINT {
                        // Move the cursor to the next line so that the prompt doesn't overwrite the `^C`.
                        println!();
                    }

                    libc::tcsetpgrp(libc::STDIN_FILENO, libc::getpgrp());
                    CURRENT_CHILD_PID.store(0, Ordering::SeqCst);

                    if libc::WIFSTOPPED(status) {
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
    }
    Ok(String::new())
}
