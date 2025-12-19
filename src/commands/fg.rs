use std::sync::atomic::Ordering;

use crate::{
    c,
    commands::jobs::{self, Job},
};

pub const USAGE: &str = "Usage:\tfg [[%]<JOB_ID>]";

pub fn fg(args: &[String], jobs: &mut Vec<Job>, current: &mut usize, previous: &mut usize) -> Result<String, String> {
    jobs::check_background_jobs(jobs, current, previous);

    let job_id = if args.len() < 2 {
        // Default to the last job if no ID provided.
        if let Some(last) = jobs.last() {
            last.id
        } else {
            return Err("Current: no such job".to_string());
        }
    } else {
        let arg = &args[1];
        let id_str = if arg.starts_with('%') { &arg[1..] } else { arg };
        id_str
            .parse::<usize>()
            .map_err(|_| format!("Invalid job ID: {}", arg))?
    };

    // We need the index so we can remove it later if it finishes.
    let index = jobs
        .iter()
        .position(|j| j.id == job_id)
        .ok_or_else(|| format!("No such job ID: {}", job_id))?;

    let pid = jobs[index].pid;
    let command_text = jobs[index].command.clone();
    *previous = *current;
    *current = job_id;

    // Print the command being brought to foreground
    println!("{}", command_text);

    // Setup signal forwarding
    c::CURRENT_CHILD_PID.store(pid, Ordering::SeqCst);

    // Send SIGCONT (in case the job was stopped).
    unsafe {
        c::kill(pid, c::SIGCONT);
    }

    // Wait (blocking).
    let mut status = 0;
    unsafe {
        c::waitpid(pid, &mut status, c::WUNTRACED);
    }

    // Teardown signal forwarding
    c::CURRENT_CHILD_PID.store(0, Ordering::SeqCst);

    if c::w_if_stopped(status) {
        // CASE A: Paused again (Ctrl+Z).
        if let Some(job) = jobs.get_mut(index) {
            job.state = crate::commands::jobs::State::Stopped;
        }

        println!("\n[{}]+\tStopped\t\t{}", job_id, command_text);
    } else {
        // CASE B: Finished or killed.
        jobs.remove(index);
        if job_id == *current {
            *current = *previous;
            *previous = 0;
        } else if job_id == *previous {
            *previous = 0;
        }
    }

    Ok(String::new())
}
