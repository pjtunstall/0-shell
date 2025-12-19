use std::sync::atomic::Ordering;

use crate::{
    c::{self, *},
    commands::jobs::{self, Job},
};

pub const USAGE: &str = "Usage:\tfg [%[+|-|%%|<JOB_ID>]]";

pub fn fg(
    args: &[String],
    jobs: &mut Vec<Job>,
    current: &mut usize,
    previous: &mut usize,
) -> Result<String, String> {
    jobs::check_background_jobs(jobs, current, previous);

    let job_id = if args.len() < 2 {
        if let Some(last) = jobs.last() {
            last.id
        } else {
            return Err("Current: no such job".to_string());
        }
    } else {
        let arg = &args[1];
        jobs::resolve_jobspec(arg, *current, *previous)?
    };

    let index = jobs
        .iter()
        .position(|j| j.id == job_id)
        .ok_or_else(|| format!("No such job ID: {}", job_id))?;

    let pid = jobs[index].pid;
    let command_text = jobs[index].command.clone();

    *previous = *current;
    *current = job_id;

    println!("{}", command_text);

    let mut status: i32 = 0;

    unsafe {
        c::tcsetpgrp(STDIN_FILENO, pid);
        CURRENT_CHILD_PID.store(pid, Ordering::SeqCst);
        c::kill(pid, SIGCONT);
        c::waitpid(pid, &mut status, WUNTRACED);
        c::tcsetpgrp(STDIN_FILENO, c::getpgrp());
        CURRENT_CHILD_PID.store(0, Ordering::SeqCst);
    }

    if c::w_if_stopped(status) {
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
