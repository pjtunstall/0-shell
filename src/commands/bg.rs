use std::collections::HashSet;

use crate::{
    c::{self, SIGCONT},
    commands::jobs::{self, Job, State},
    error,
};

pub const USAGE: &str = "Usage:\tbg [%[+|-|%%|<JOB_ID>]]...";

pub fn bg(
    input: &[String],
    jobs: &mut Vec<Job>,
    current: &mut usize,
    previous: &mut usize,
) -> Result<String, String> {
    jobs::check_background_jobs(jobs, current, previous);

    let mut failures = String::new();
    let mut successes = Vec::new();

    let mut failure_count: usize = 0;
    let mut success_count: usize = 0;

    let mut target_ids = HashSet::new();

    if input.len() < 2 {
        let fallback = if *current > 0 {
            Some(*current)
        } else if *previous > 0 {
            Some(*previous)
        } else {
            None
        };

        if let Some(id) = fallback {
            target_ids.insert(id);
        } else {
            return Err("Current: no such job".to_string());
        }
    } else {
        for item in &input[1..] {
            match jobs::resolve_jobspec(item, *current, *previous) {
                Ok(id) => {
                    target_ids.insert(id);
                }
                Err(e) => {
                    failures.push_str(&format!("{}\n", e));
                    failure_count += 1
                }
            }
        }
    }

    for job in jobs.iter_mut() {
        if target_ids.contains(&job.id) {
            if matches!(job.state, State::Stopped) {
                unsafe {
                    // Send SIGCONT to the process group (negative PID)
                    // so that all members of a pipeline resume together.
                    c::kill(-job.pid, SIGCONT);
                }
                job.state = State::Running;
                if job.id != *current {
                    *previous = *current;
                    *current = job.id;
                }
                success_count += 1;
                successes.push(&*job);
            } else {
                failures.push_str(&format!("Job is not stopped: {}\n", job.id));
                failure_count += 1;
            }
            target_ids.remove(&job.id);
        }
    }

    for id in target_ids {
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

        let mut current = 0;
        let mut previous = 0;
        let result = bg(&input, &mut jobs, &mut current, &mut previous);

        assert!(result.is_ok());
        assert!(matches!(jobs[0].state, State::Running));
        assert!(matches!(jobs[1].state, State::Running));
        assert_eq!(current, 1);
        assert_eq!(previous, 0);
    }

    #[test]
    fn test_bg_no_args_resumes_current() {
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
                state: State::Stopped,
                command: "sleep 200".to_string(),
            },
        ];

        let mut current = 2;
        let mut previous = 1;
        let input = vec!["bg".to_string()];

        let result = bg(&input, &mut jobs, &mut current, &mut previous);

        assert!(result.is_ok());
        assert!(matches!(jobs[1].state, State::Running));
        assert_eq!(current, 2);
        assert_eq!(previous, 1);
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

        let mut current = 0;
        let mut previous = 0;
        let result = bg(&input, &mut jobs, &mut current, &mut previous);

        assert!(result.is_ok());
        assert!(matches!(jobs[0].state, State::Running));
        assert_eq!(current, 1);
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

        let mut current = 0;
        let mut previous = 0;
        let _ = bg(&input, &mut jobs, &mut current, &mut previous);

        assert!(matches!(jobs[0].state, State::Stopped));
        assert_eq!(current, 0);
        assert_eq!(previous, 0);
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

        let mut current = 0;
        let mut previous = 0;
        let result = bg(&input, &mut jobs, &mut current, &mut previous);
        let output = result.expect("bg command failed");
        let parts: Vec<&str> = output.split(':').collect();
        let success_count: usize = parts[0].parse().expect("parsing success count failed");
        let failure_count: usize = parts[1].parse().expect("parsing failure count failed");

        assert_eq!(success_count, 1);
        assert_eq!(failure_count, 3);
    }
}
