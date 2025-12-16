pub mod cat;
pub mod cd;
pub mod cp;
pub mod echo;
pub mod exit;
pub mod fg;
pub mod jobs;
pub mod ls;
pub mod man;
pub mod mkdir;
pub mod mv;
pub mod pwd;
pub mod rm;
pub mod sleep;
pub mod touch;

use crate::commands::jobs::Job;
use crate::{error, worker};

// This is the command runner that's called by the parent process.
// We accept the `jobs` `Vec` here to pass it down to the `jobs`
// function for viewing or to 'worker' (for updating if a process
// is stopped).
pub fn run_command(args: &[String], jobs: &mut Vec<Job>) {
    if args.is_empty() {
        return;
    }

    let command = args[0].as_str();

    let result = match command {
        // State modifiers (built-in).
        "cd" => cd::cd(args),
        "exit" => exit::exit(args),

        // Lightweight utilities (built-in).
        "echo" => echo::echo(args),
        "pwd" => pwd::pwd(args),

        // Job control (built-in).
        "jobs" => jobs::jobs(args, jobs),
        "fg" => fg::fg(args, jobs),

        // External utilities.
        // We delegate these to a child process so they can be stopped/killed without crashing the main shell.
        "cat" | "cp" | "ls" | "mkdir" | "man" | "mv" | "rm" | "sleep" | "touch" => {
            worker::launch_worker_process(args, jobs)
        }

        _ => Err(format!("command not found: {}", command)),
    };

    match result {
        Ok(ok) => {
            if !ok.is_empty() {
                // Some commands (like `ls`` or `jobs`) return a string to print.
                // Others (like `cd`) return an empty string.
                print!("{}", &ok);
            }
        }
        Err(err) => error::handle_error(command, err),
    }
}

// This function runs inside the child process spawned by `launch_worker_process`.
pub fn run_command_as_worker(args: &[String]) {
    // args[0] is the executable path
    // args[1] is "--internal-worker"
    // args[2] is the actual command (e.g. "ls")
    if args.len() < 3 {
        error::red_println("0-shell: internal worker error: missing command argument");
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
        _ => Err(format!("command not found: {}", command)),
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
