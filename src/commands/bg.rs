use std::{collections::HashSet, io};

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
            return Err(String::from("Current: no such job"));
        }
    } else {
        for item in &input[1..] {
            match jobs::resolve_jobspec(item, *current, *previous) {
                Ok(id) => {
                    target_ids.insert(id);
                }
                Err(e) => {
                    failures.push_str(&format!("{e}\n"));
                    failure_count += 1
                }
            }
        }
    }

    for job in jobs.iter_mut() {
        let jid = job.jid;
        let pgid = job.pgid;

        if target_ids.contains(&jid) {
            if matches!(job.state, State::Stopped) {
                // Send SIGCONT to the process group (negative PID)
                // so that all members of a pipeline resume together.
                if let Err(err) = resumer.resume(pgid) {
                    failures.push_str(&format!(
                        "Failed to resume job {jid} (pid {pgid}): {err}\n"
                    ));
                    failure_count += 1;
                    target_ids.remove(&jid);
                    continue;
                }
                job.state = State::Running;
                if jid != *current {
                    *previous = *current;
                    *current = jid;
                }
                success_count += 1;
                successes.push(&*job);
            } else {
                failures.push_str(&format!("Job is not stopped: {jid}\n"));
                failure_count += 1;
            }
            target_ids.remove(&jid);
        }
    }

    for id in target_ids {
        failures.push_str(&format!("No such job ID: {id}\n"));
        failure_count += 1;
    }

    for job in successes {
        println!("[{}]+\t{} &", job.jid, job.command);
    }

    if !failures.is_empty() {
        error::red_println(&failures.trim_end_matches('\n'));
    }

    Ok(format!("{success_count}:{failure_count}"))
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
                jid: 1,
                pgid: 101,
                state: State::Stopped,
                command: String::from("sleep 100"),
            },
            Job {
                jid: 2,
                pgid: 102,
                state: State::Running,
                command: String::from("ls"),
            },
        ];

        let input = vec![String::from("bg"), String::from("1")];

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
                jid: 1,
                pgid: 101,
                state: State::Stopped,
                command: String::from("sleep 100"),
            },
            Job {
                jid: 2,
                pgid: 102,
                state: State::Stopped,
                command: String::from("sleep 200"),
            },
        ];

        let mut current = 2;
        let mut previous = 1;
        let input = vec![String::from("bg")];

        let result = bg_with_resumer(&input, &mut jobs, &mut current, &mut previous, &NoopResumer);

        assert!(result.is_ok());
        assert!(matches!(jobs[1].state, State::Running));
        assert_eq!(current, 2);
        assert_eq!(previous, 1);
    }

    #[test]
    fn test_bg_supports_percent_syntax() {
        let mut jobs = vec![Job {
            jid: 1,
            pgid: 101,
            state: State::Stopped,
            command: String::from("sleep 100"),
        }];

        let input = vec![String::from("bg"), String::from("%1")];

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
            jid: 1,
            pgid: 101,
            state: State::Stopped,
            command: String::from("sleep"),
        }];
        let input = vec![String::from("bg"), String::from("999")];

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
                jid: 1,
                pgid: 101,
                state: State::Stopped,
                command: String::from("sleep"),
            },
            Job {
                jid: 2,
                pgid: 102,
                state: State::Running,
                command: String::from("ls"),
            },
        ];
        let input = vec![
            String::from("bg"),
            String::from("1"),
            String::from("not_a_job_id"),
            String::from("2"),
            String::from("also_not_a_job_id"),
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
