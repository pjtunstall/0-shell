use std::time::Duration;

use crate::{
    c::{self, SIGCONT, SIGTERM},
    commands::jobs::{self, Job, State},
};

pub const USAGE: &str = "Usage:\tkill <PID>|%[+|-|%%|<JOB_ID>]";

pub fn kill(input: &[String], jobs: &mut Vec<Job>, current: &mut usize, previous: &mut usize) -> Result<String, String> {
    jobs::check_background_jobs(jobs, current, previous);

    if input.len() > 2 {
        return Err(format!("Too many arguments\n{}", USAGE));
    }

    if input.len() < 2 {
        return Err(format!("Not enough arguments\n{}", USAGE));
    }

    let arg = &input[1];
    let pid_to_kill: i32;
    let mut is_stopped = false;

    if arg.starts_with('%') {
        let job_id = jobs::resolve_jobspec(arg, *current, *previous)?;

        if let Some(job) = jobs.iter().find(|j| j.id == job_id) {
            pid_to_kill = job.pid;
            if matches!(job.state, State::Stopped) {
                is_stopped = true;
            }
        } else {
            return Err(format!("No such job ID: {}", arg));
        }
    } else {
        pid_to_kill = arg
            .parse::<i32>()
            .map_err(|e| format!("Failed to parse PID: {}\n{}", e, arg))?;

        if pid_to_kill <= 0 {
            return Err("PID must be positive".to_string());
        }

        // Note if it's one of our tracked jobs (to wake stopped jobs).
        if let Some(job) = jobs.iter().find(|j| j.pid == pid_to_kill) {
            if matches!(job.state, State::Stopped) {
                is_stopped = true;
            }
        }
    }

    unsafe {
        // If the job is running, this kills it immediately.
        // If the job is stopped, the signal is queued.
        c::kill(pid_to_kill, SIGTERM);

        // Restart a stopped job so that it can received the queued signal to terminate.
        if is_stopped {
            c::kill(pid_to_kill, SIGCONT);
        }
    }

    // Surface termination promptly instead of waiting for the next builtin call.
    // Poll briefly so we don't block indefinitely if the process ignores SIGTERM.
    for _ in 0..5 {
        jobs::check_background_jobs(jobs, current, previous);
        if jobs.iter().all(|j| j.pid != pid_to_kill) {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }

    Ok(String::new())
}
