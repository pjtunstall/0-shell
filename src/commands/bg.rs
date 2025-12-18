use std::collections::HashSet;

use crate::{
    c::{self, SIGCONT},
    commands::jobs::{self, Job, State},
    error,
};

pub const USAGE: &str = "Usage:\tbg [%]<JOB_ID>...";

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
        let clean_item = if item.starts_with('%') {
            &item[1..]
        } else {
            item
        };

        match clean_item.parse::<usize>() {
            Ok(id) => {
                target_ids.insert(id);
            }
            Err(e) => {
                // Command prefix removed
                failures.push_str(&format!("Failed to parse job ID: {}\n", e));
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
                successes.push(&*job);
            } else {
                // Command prefix removed
                failures.push_str(&format!("Job {} is not stopped\n", job.id));
                failure_count += 1;
            }
            target_ids.remove(&job.id);
        }
    }

    for id in target_ids {
        // Command prefix removed
        failures.push_str(&format!("No such job ID: {}\n", id));
        failure_count += 1;
    }

    for job in successes {
        println!("[{}]+\t{} &", job.id, job.command);
    }

    if !failures.is_empty() {
        error::red_println(&failures.trim_end_matches('\n'));
    }

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
            "job id 1 should move to running"
        );
        assert!(
            matches!(jobs[1].state, State::Running),
            "job id 2 should stay running"
        );
    }

    #[test]
    fn test_bg_supports_percent_syntax() {
        let mut jobs = vec![Job {
            id: 1,
            pid: 101,
            state: State::Stopped,
            command: "sleep 100".to_string(),
        }];

        let input = vec!["bg".to_string(), "%1".to_string()];

        let result = bg(&input, &mut jobs);

        assert!(result.is_ok());
        assert!(
            matches!(jobs[0].state, State::Running),
            "job id 1 should move to running with % syntax"
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
            "state should not have changed"
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

        assert_eq!(success_count, 1, "should have 1 successful bg");
        assert_eq!(failure_count, 3, "should have 3 failed bg attempts");
    }
}
