use std::{io, time::Duration};

use crate::commands::jobs::{self, Job, State};

pub const USAGE: &str = "Usage:\tkill <PID>|%[+|-|%%|<JOB_ID>]";

pub fn kill(
    input: &[String],
    jobs: &mut Vec<Job>,
    current: &mut usize,
    previous: &mut usize,
) -> Result<String, String> {
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
        let job_id = jobs::resolve_jobspec_or_pid(arg, *current, *previous)?;

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
            return Err(String::from("PID must be positive"));
        }

        if let Some(job) = jobs.iter().find(|j| j.pid == pid_to_kill) {
            if matches!(job.state, State::Stopped) {
                is_stopped = true;
            }
        }
    }

    unsafe {
        // Use negative PID to target the process group leader and all its children.
        // If the job is running, this kills it immediately.
        // If the job is stopped, the signal is queued.
        if libc::kill(-pid_to_kill, libc::SIGTERM) == -1 {
            let err = io::Error::last_os_error();
            return Err(format!("Failed to kill {}: {}", pid_to_kill, err));
        }

        // Restart a stopped job so that it can receive the queued signal to terminate.
        if is_stopped {
            if libc::kill(-pid_to_kill, libc::SIGCONT) == -1 {
                let err = io::Error::last_os_error();
                return Err(format!("Failed to resume {} for termination: {}", pid_to_kill, err));
            }
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
