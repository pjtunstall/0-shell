use std::sync::atomic::Ordering;

use crate::{c, commands::jobs::Job};

pub const USAGE: &str = "Usage: fg [ID]";

pub fn fg(args: &[String], jobs: &mut Vec<Job>) -> Result<String, String> {
    let job_id = if args.len() < 2 {
        // Default to the last job if no ID provided (the "+" job).
        if let Some(last) = jobs.last() {
            last.id
        } else {
            return Err("Current: no such job".to_string());
        }
    } else {
        args[1]
            .parse::<usize>()
            .map_err(|_| "Arguments must be job IDs".to_string())?
    };

    // We need the index so we can remove it later if it finishes.
    let index = jobs
        .iter()
        .position(|j| j.id == job_id)
        .ok_or_else(|| format!("No such job: {}", job_id))?;

    let pid = jobs[index].pid;
    let command_text = jobs[index].command.clone();

    println!("{}", command_text);

    c::CURRENT_CHILD_PID.store(pid, Ordering::Relaxed);

    unsafe {
        c::kill(pid, c::SIGCONT);
    }

    let mut status = 0;
    unsafe {
        c::waitpid(pid, &mut status, c::WUNTRACED);
    }

    // We (the parent process) wake up.
    // We reset the foreground child to none ...
    c::CURRENT_CHILD_PID.store(0, Ordering::Relaxed);

    // ... and check why the child that was running returned control to us.
    if c::w_if_stopped(status) {
        // CASE A: paused again (Ctrl+Z).
        if let Some(job) = jobs.get_mut(index) {
            job.state = crate::commands::jobs::State::Stopped;
        }

        println!("\n[{}]+\tStopped\t\t{}", job_id, command_text);
    } else {
        // CASE B: killed or finished.
        // The process is dead. Remove it from our bookkeeping.
        jobs.remove(index);
    }

    Ok(String::new())
}
