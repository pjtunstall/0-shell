use std::collections::HashSet;
use std::io;

use crate::{
    commands::jobs::{self, Job, State},
    error,
};

trait Resumer {
    fn resume(&self, pgid: i32) -> Result<(), io::Error>;
}

struct RealResumer;

impl Resumer for RealResumer {
    fn resume(&self, pgid: i32) -> Result<(), io::Error> {
        let res = unsafe { libc::kill(-pgid, libc::SIGCONT) };
        if res == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

pub const USAGE: &str = "Usage:\tbg [jobspec ...]";

pub fn bg(
    input: &[String],
    jobs: &mut Vec<Job>,
    current: &mut usize,
    previous: &mut usize,
) -> Result<String, String> {
    bg_with_resumer(input, jobs, current, previous, &RealResumer)
}

fn bg_with_resumer(
    input: &[String],
    jobs: &mut Vec<Job>,
    current: &mut usize,
    previous: &mut usize,
    resumer: &dyn Resumer,
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
            match jobs::resolve_jobspec_or_pid(item, *current, *previous) {
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
                // Send SIGCONT to the process group (negative PID)
                // so that all members of a pipeline resume together.
                if let Err(err) = resumer.resume(job.pid) {
                    failures.push_str(&format!(
                        "Failed to resume job {} (pid {}): {}\n",
                        job.id, job.pid, err
                    ));
                    failure_count += 1;
                    target_ids.remove(&job.id);
                    continue;
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

    struct NoopResumer;

    impl Resumer for NoopResumer {
        fn resume(&self, _pgid: i32) -> Result<(), io::Error> {
            Ok(())
        }
    }

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
        let result = bg_with_resumer(&input, &mut jobs, &mut current, &mut previous, &NoopResumer);

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

        let result = bg_with_resumer(&input, &mut jobs, &mut current, &mut previous, &NoopResumer);

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
        let result = bg_with_resumer(&input, &mut jobs, &mut current, &mut previous, &NoopResumer);

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
        let _ = bg_with_resumer(&input, &mut jobs, &mut current, &mut previous, &NoopResumer);

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
        let result = bg_with_resumer(&input, &mut jobs, &mut current, &mut previous, &NoopResumer);
        let output = result.expect("bg command failed");
        let parts: Vec<&str> = output.split(':').collect();
        let success_count: usize = parts[0].parse().expect("parsing success count failed");
        let failure_count: usize = parts[1].parse().expect("parsing failure count failed");

        assert_eq!(success_count, 1);
        assert_eq!(failure_count, 3);
    }
}
