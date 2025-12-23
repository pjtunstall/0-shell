use std::{io, sync::atomic::Ordering};

use crate::commands::{
    fg::CURRENT_CHILD_PID,
    jobs::{Job, State},
};

pub fn run_parent(
    args: &[String],
    jobs: &mut Vec<Job>,
    is_background_launch: bool,
    current: &mut usize,
    previous: &mut usize,
    child_pid: i32,
) -> Result<String, String> {
    unsafe {
        // Make the child the leader of a new process group. We place this line
        // in both parent and child code because we don't know which the OS will
        // choose to run first.
        libc::setpgid(child_pid, child_pid);

        match is_background_launch {
            true => {
                let id = jobs.len() + 1;
                let cmd_str = display_command(args);
                jobs.push(Job::new(id, child_pid, cmd_str, State::Running));
                *previous = *current;
                *current = id;
                return Ok(format!("[{}] {}\n", id, child_pid));
            }
            false => {
                // Snapshot the shell's current terminal settings in case a
                // child process crashes, leaving the terminal in some other
                // state (e.g., cooked vs. raw mode, or perhaps a different
                // flavor of raw to that of my shell).
                let mut shell_termios: libc::termios = std::mem::zeroed();
                libc::tcgetattr(libc::STDIN_FILENO, &mut shell_termios);

                // Hand terminal control to the foreground child's group.
                // It will now be be the recipient of any SIGINT or SIGTSTP.
                libc::tcsetpgrp(libc::STDIN_FILENO, child_pid);
                CURRENT_CHILD_PID.store(child_pid, Ordering::SeqCst);

                let status = wait_for_foreground_child(child_pid)?;

                // If the child was terminated by SIGINT, the cursor to the next
                // line so that the prompt doesn't overwrite the `^C`.
                if libc::WIFSIGNALED(status) && libc::WTERMSIG(status) == libc::SIGINT {
                    println!();
                }

                // Reclaim the terminal for the shell. At this point, since the
                // child is in the foreground, we're in the background. So, we
                // receive a SIGTTOU when we try to take back control, but
                // that's ok. In `repl::repl`, we set the action to
                // "ignore", so the SIGTTOU doesn't stop us.
                libc::tcsetpgrp(libc::STDIN_FILENO, libc::getpgrp());
                CURRENT_CHILD_PID.store(0, Ordering::SeqCst);

                // Force terminal back to the shell's settings. `TCSADRAIN`
                // ensures all output has been transmitted before the change.
                libc::tcsetattr(libc::STDIN_FILENO, libc::TCSADRAIN, &shell_termios);

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

// Block the shell process until the specific foreground child `child_pid`
// changes state. This handles both termination (Ctrl+C) and suspension (Ctrl+Z).
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
