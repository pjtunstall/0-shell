use std::sync::atomic::Ordering;

use crate::{
    c::CURRENT_CHILD_PID,
    commands::jobs::{self, Job},
};

pub const USAGE: &str = "Usage:\tfg [jobspec]";

pub fn fg(
    args: &[String],
    jobs: &mut Vec<Job>,
    current: &mut usize,
    previous: &mut usize,
) -> Result<String, String> {
    jobs::check_background_jobs(jobs, current, previous);

    let job_id = if args.len() < 2 {
        if let Some(last) = jobs.last() {
            last.jid
        } else {
            return Err(String::from("Current: no such job"));
        }
    } else {
        let arg = &args[1];
        jobs::resolve_jobspec(arg, *current, *previous)?
    };

    let index = jobs
        .iter()
        .position(|j| j.jid == job_id)
        .ok_or_else(|| format!("No such job ID: {job_id}"))?;

    let pgid = jobs[index].pgid;
    let command_text = jobs[index].command.clone();

    *previous = *current;
    *current = job_id;

    println!("{}", command_text);

    let mut status: i32 = 0;

    unsafe {
        libc::tcsetpgrp(libc::STDIN_FILENO, pgid);
        CURRENT_CHILD_PID.store(pgid, Ordering::SeqCst);
        libc::kill(pgid, libc::SIGCONT);
        loop {
            let res = libc::waitpid(pgid, &mut status, libc::WUNTRACED);
            if res == pgid {
                break;
            }
            if res == -1 {
                let err = std::io::Error::last_os_error();
                if err.raw_os_error() == Some(libc::EINTR) {
                    continue;
                }
                return Err(format!("waitpid failed: {err}"));
            }
            return Err(format!("waitpid returned unexpected pid: {res}"));
        }
        libc::tcsetpgrp(libc::STDIN_FILENO, libc::getpgrp());
        CURRENT_CHILD_PID.store(0, Ordering::SeqCst);
    }

    if libc::WIFSTOPPED(status) {
        if let Some(job) = jobs.get_mut(index) {
            job.state = crate::commands::jobs::State::Stopped;
        }

        println!("\n[{}]+\tStopped\t\t{}", job_id, command_text);
    } else {
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

#[cfg(test)]
mod tests {
    use super::fg;
    use crate::{commands::jobs::Job, string_vec};

    #[test]
    fn fg_errors_when_no_jobs_and_no_args() {
        let mut jobs = Vec::<Job>::new();
        let mut current = 0;
        let mut previous = 0;
        let input = string_vec!["fg"];

        let result = fg(&input, &mut jobs, &mut current, &mut previous);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Current: no such job");
    }

    #[test]
    fn fg_errors_when_job_not_found() {
        let mut jobs = Vec::<Job>::new();
        let mut current = 0;
        let mut previous = 0;
        let input = string_vec!["fg", "5"];

        let result = fg(&input, &mut jobs, &mut current, &mut previous);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No such job ID: 5");
    }
}
