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

#[cfg(unix)]
use std::{process::Command, sync::atomic::Ordering};

use crate::{
    c::{self, *},
    commands::jobs::Job,
    error, worker,
};

// This is the command runner that's called by the parent process. We accept (a mutable reference to) the `Vec` of jobs here to pass it down to the `jobs` function for viewing or to 'worker' (so that we can update it if a process has stopped).
// By contrast, `run_command_as_worker` below is the command runner launched by child processes (jobs).
pub fn run_command(
    args: &[String],
    jobs: &mut Vec<Job>,
    current: &mut usize,
    previous: &mut usize,
) {
    if args.is_empty() {
        return;
    }

    let (clean_args, is_background) = if let Some(last) = args.last() {
        if last == "&" {
            (&args[..args.len() - 1], true) // Slice off the '&'.
        } else {
            (args, false)
        }
    } else {
        (args, false)
    };

    if clean_args.is_empty() {
        if is_background {
            error::red_println("0-shell: Syntax error near unexpected token `&'");
        }
        return;
    }

    let command = clean_args[0].as_str();

    let result = match command {
        // State modifiers (built-in).
        "cd" => cd::cd(args),
        "exit" => exit::exit(args),

        // Lightweight utilities (built-in).
        "echo" => echo::echo(args),
        "pwd" => pwd::pwd(args),

        // Job control (built-in).
        "bg" => bg::bg(args, jobs, current, previous),
        "fg" => fg::fg(args, jobs, current, previous),
        "jobs" => jobs::jobs(args, jobs, current, previous),
        "kill" => kill::kill(args, jobs, current, previous),

        // Re-implemented utilities, launched as child processes so they can be stopped/killed without crashing the main shell.
        "cat" | "cp" | "ls" | "mkdir" | "man" | "mv" | "rm" | "sleep" | "touch" => {
            worker::launch_job(clean_args, jobs, is_background, current, previous)
        }

        // External utilities, likewise spawned as children.
        _ => launch_external(clean_args, jobs, is_background, current, previous),
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

fn launch_external(
    args: &[String],
    jobs: &mut Vec<Job>,
    is_background: bool,
    current: &mut usize,
    previous: &mut usize,
) -> Result<String, String> {
    let (program, prog_args) = args
        .split_first()
        .ok_or_else(|| "Missing command".to_string())?;

    let mut cmd = Command::new(program);
    cmd.args(prog_args);

    if is_background {
        let child = cmd.spawn().map_err(|e| e.to_string())?;
        let pid = child.id() as i32;
        let id = jobs.len() + 1;
        let command_string = args.join(" ");
        jobs.push(Job::new(id, pid, command_string, jobs::State::Running));
        *previous = *current;
        *current = id;
        println!("[{}] {}", id, pid);
        Ok(String::new())
    } else {
        let child = cmd.spawn().map_err(|e| e.to_string())?;
        let pid = child.id() as i32;

        unsafe {
            CURRENT_CHILD_PID.store(pid, Ordering::SeqCst);
            let mut status = 0;

            // Blocks until the child process either dies or stops (`WUNTRACED`).
            c::waitpid(pid, &mut status, WUNTRACED);
            CURRENT_CHILD_PID.store(0, std::sync::atomic::Ordering::SeqCst);

            if c::w_if_stopped(status) {
                let id = jobs.len() + 1;
                let command_string = args.join(" ");
                let new_job = Job::new(id, pid, command_string.clone(), jobs::State::Stopped);
                jobs.push(new_job);
                *previous = *current;
                *current = id;
                println!("\n[{}]+\tStopped\t\t{}", id, command_string);
            }
        }
        Ok(String::new())
    }
}

// This function runs inside the child process spawned by `launch_worker_process`.
pub fn run_command_as_worker(args: &[String]) {
    // args[0] is the executable path
    // args[1] is "--internal-worker"
    // args[2] is the actual command (e.g. "ls")
    if args.len() < 3 {
        error::red_println("0-shell: Internal worker error: missing command argument");
        return;
    }

    let command = args[2].as_str();
    let clean_args = &args[2..]; // Slice starting from the command name.

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
