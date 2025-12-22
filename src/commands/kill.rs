use std::{io, time::Duration};

use crate::commands::jobs::{self, Job, State};

pub const USAGE: &str = "Usage:\tkill <LEADER_PID>|%[+|-|%%|<PGID>]";

pub fn kill(
    input: &[String],
    jobs: &mut Vec<Job>,
    current: &mut usize,
    previous: &mut usize,
) -> Result<String, String> {
    jobs::check_background_jobs(jobs, current, previous);

    if input.len() > 2 {
        return Err(format!("Too many arguments\n{USAGE}"));
    }

    if input.len() < 2 {
        return Err(format!("Not enough arguments\n{USAGE}"));
    }

    let arg = &input[1];
    // May represent the PGID of the group or the PID of the group leader,
    // depending on whether the value passed to `job` is prefixed with `%` (in
    // which case `-pid_to_kill` is passed to `libc::kill`.)
    let pid_to_kill: i32;
    let mut is_stopped = false;

    if arg.starts_with('%') {
        let job_id = jobs::resolve_jobspec(arg, *current, *previous)?;

        if let Some(job) = jobs.iter().find(|j| j.pgid == job_id) {
            pid_to_kill = job.leader_pid;
            if matches!(job.state, State::Stopped) {
                is_stopped = true;
            }
        } else {
            return Err(format!("No such job ID: {arg}"));
        }
    } else {
        pid_to_kill = arg
            .parse::<i32>()
            .map_err(|e| format!("Failed to parse ID: {e}\r\n{arg}\r\n{USAGE}"))?;

        if pid_to_kill <= 0 {
            return Err(String::from("ID must be positive"));
        }

        if let Some(job) = jobs.iter().find(|j| j.leader_pid == pid_to_kill) {
            if matches!(job.state, State::Stopped) {
                is_stopped = true;
            }
        }
    }

    unsafe {
        // If the job is running, this kills it immediately. If the job is
        // stopped, the signal is queued. Use a negative value for the first
        // argument to target the process group leader and all its children;
        // `libc::kill` interprets a positive value as the PID of thegroup
        // leader.
        if libc::kill(-pid_to_kill, libc::SIGTERM) == -1 {
            let err = io::Error::last_os_error();
            return Err(format!("Failed to kill {pid_to_kill}: {err}"));
        }

        // Restart a stopped job so that it can receive the queued signal to
        // terminate.
        if is_stopped {
            if libc::kill(-pid_to_kill, libc::SIGCONT) == -1 {
                let err = io::Error::last_os_error();
                return Err(format!(
                    "Failed to resume {pid_to_kill} for termination: {err}"
                ));
            }
        }
    }

    // Surface termination promptly instead of waiting for the next builtin call.
    // Poll briefly so we don't block indefinitely if the process ignores SIGTERM.
    for _ in 0..5 {
        jobs::check_background_jobs(jobs, current, previous);
        if jobs.iter().all(|j| j.leader_pid != pid_to_kill) {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }

    Ok(String::new())
}
