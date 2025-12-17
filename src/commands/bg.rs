use std::collections::HashSet;

use crate::{
    c::{self, SIGCONT},
    commands::jobs::{self, Job, State},
    error,
};

pub const USAGE: &str = "bg [JOB...]";

pub fn bg(input: &[String], jobs: &mut Vec<Job>) -> Result<String, String> {
    jobs::check_background_jobs(jobs);

    if input.len() < 2 {
        return Err(format!("Not enough arguments\n{}", USAGE));
    }

    let mut failures = String::new();
    let mut successes = Vec::new();

    let mut failure_count: usize = 0;
    let mut success_count: usize = 0;

    let mut target_ids = HashSet::new();

    for item in &input[1..] {
        match item.parse::<usize>() {
            Ok(id) => {
                target_ids.insert(id);
            }
            Err(e) => {
                failures.push_str(&format!("bg: Failed to parse job ID: {}\n", e));
                failure_count += 1
            }
        }
    }

    for job in jobs.iter_mut() {
        if target_ids.contains(&job.id) {
            if matches!(job.state, State::Stopped) {
                unsafe {
                    c::kill(job.pid, SIGCONT);
                }
                job.state = State::Running;
                success_count += 1;

                // Reborrow; fine, thanks to NLL, as long as we don't use the current mutable reference to a job again after this iteration.
                successes.push(&*job);
            } else {
                failures.push_str(&format!("bg: Job {} is not stopped\n", job.id));
                failure_count += 1;
            }
            target_ids.remove(&job.id);
        }
    }

    // Report any requested job IDs that weren't found
    for id in target_ids {
        failures.push_str(&format!("bg: No job with ID: {}\n", id));
        failure_count += 1;
    }

    let output = jobs::format_jobs(&successes, false);
    print!("{}", output);
    if !failures.is_empty() {
        error::red_println(&failures.trim_end_matches('\n'));
    }

    // As a stopgap till I have time to restructure so that `bg` (and `ls`) can return multiple ok-like and error-like items, I'm repurposing this `Ok` return value to allow me to test the number of successes and failures.
    Ok(format!("{}:{}", success_count, failure_count))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bg_updates_stopped_jobs() {
        let mut jobs = vec![
            Job {
                id: 1,
                pid: 101,
                state: State::Stopped,
                command: "sleep 100".to_string(),
            },
            Job {
                id: 2,
                pid: 102,
                state: State::Running,
                command: "ls".to_string(),
            },
        ];

        let input = vec!["bg".to_string(), "1".to_string()];

        let result = bg(&input, &mut jobs);

        assert!(result.is_ok());
        assert!(
            matches!(jobs[0].state, State::Running),
            "Job ID 1 should move to Running"
        );
        assert!(
            matches!(jobs[1].state, State::Running),
            "Job ID 2 should stay Running"
        );
    }

    #[test]
    fn test_bg_ignores_missing_pids() {
        let mut jobs = vec![Job {
            id: 1,
            pid: 101,
            state: State::Stopped,
            command: "sleep".to_string(),
        }];
        let input = vec!["bg".to_string(), "999".to_string()];

        let _ = bg(&input, &mut jobs);

        assert!(
            matches!(jobs[0].state, State::Stopped),
            "Should not have changed"
        );
    }

    #[test]
    fn test_generates_correct_number_of_failure_messages() {
        let mut jobs = vec![
            Job {
                id: 1,
                pid: 101,
                state: State::Stopped,
                command: "sleep".to_string(),
            },
            Job {
                id: 2,
                pid: 102,
                state: State::Running,
                command: "ls".to_string(),
            },
        ];
        let input = vec![
            "bg".to_string(),
            "1".to_string(),
            "not_a_job_id".to_string(),
            "2".to_string(),
            "also_not_a_job_id".to_string(),
        ];

        let result = bg(&input, &mut jobs);
        let output = result.expect("bg command failed");
        let parts: Vec<&str> = output.split(':').collect();
        let success_count: usize = parts[0].parse().expect("parsing success count failed");
        let failure_count: usize = parts[1].parse().expect("parsing failure count failed");

        assert_eq!(success_count, 1, "Should have 1 successful bg");
        assert_eq!(failure_count, 3, "Should have 3 failed bg attempts"); // Two parsing failures and one job already running.
    }
}
