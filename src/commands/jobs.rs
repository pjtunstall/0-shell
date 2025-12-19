use crate::c;
use std::borrow::Borrow;

pub const USAGE: &str = "Usage:\tjobs [-lprs] [[%]<JOB_ID>...]";
const STATE_COL_WIDTH: usize = 24;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JobOptions {
    pub show_pid: bool,     // -l
    pub pid_only: bool,     // -p
    pub running_only: bool, // -r
    pub stopped_only: bool, // -s
}

impl Default for JobOptions {
    fn default() -> Self {
        Self {
            show_pid: false,
            pid_only: false,
            running_only: false,
            stopped_only: false,
        }
    }
}

pub fn jobs(
    input: &[String],
    jobs: &mut Vec<Job>,
    current: &mut usize,
    previous: &mut usize,
) -> Result<String, String> {
    check_background_jobs(jobs, current, previous);

    let mut opts = JobOptions::default();
    let mut specific_job_ids = Vec::new();

    for arg in input.iter().skip(1) {
        if arg.starts_with('-') {
            for c in arg.chars().skip(1) {
                match c {
                    'l' => opts.show_pid = true,
                    'p' => opts.pid_only = true,
                    'r' => opts.running_only = true,
                    's' => opts.stopped_only = true,
                    _ => return Err(format!("Invalid option -- '{}'\n{}", c, USAGE)),
                }
            }
        } else {
            let id_str = if arg.starts_with('%') { &arg[1..] } else { arg };
            match id_str.parse::<usize>() {
                Ok(id) => specific_job_ids.push(id),
                Err(_) => return Err(format!("Invalid job ID: {}", arg)),
            }
        }
    }

    let output = format_jobs(jobs, opts, &specific_job_ids, *current, *previous);
    Ok(output)
}

pub fn format_jobs<T: Borrow<Job>>(
    items: &[T],
    opts: JobOptions,
    filter_ids: &[usize],
    current: usize,
    previous: usize,
) -> String {
    let mut output = String::new();

    for item in items.iter() {
        let job = item.borrow();

        // 0. Filter by specific Job IDs (if user asked for specific jobs)
        if !filter_ids.is_empty() && !filter_ids.contains(&job.id) {
            continue;
        }

        // 1. Filter: -r (Running only).
        if opts.running_only && matches!(job.state, State::Stopped) {
            continue;
        }
        // 2. Filter: -s (Stopped only).
        if opts.stopped_only && matches!(job.state, State::Running) {
            continue;
        }

        // Mode: -p (PIDs only). Early exit.
        if opts.pid_only {
            output.push_str(&format!("{}\n", job.pid));
            continue;
        }

        let sign = if job.id == current {
            "+"
        } else if job.id == previous {
            "-"
        } else {
            " "
        };

        let display = JobDisplay { job, sign, opts };
        output.push_str(&format!("{}\n", display));
    }

    output
}

pub struct Job {
    pub id: usize,
    pub pid: i32,
    pub command: String,
    pub state: State,
}

impl Job {
    pub fn new(jobs_total: usize, pid: i32, command: String, state: State) -> Self {
        assert!(pid > 0, "`pid` must be positive");
        Self {
            id: jobs_total,
            pid,
            command,
            state,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum State {
    Running,
    Stopped,
    Terminated,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            State::Running => "Running",
            State::Stopped => "Stopped",
            State::Terminated => "Terminated", // Rarely seen (removed immediately)
        };
        f.write_str(s)
    }
}

pub struct JobDisplay<'a> {
    pub job: &'a Job,
    pub sign: &'a str,
    pub opts: JobOptions,
}

impl std::fmt::Display for JobDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state_str = self.job.state.to_string();

        // Append " &" only if it is currently running.
        // Stopped jobs do not get the ampersand.
        let ampersand = if matches!(self.job.state, State::Running) {
            " &"
        } else {
            ""
        };

        if self.opts.show_pid {
            write!(
                f,
                "[{}]{} {:<5} {:<width$} {}{}",
                self.job.id,
                self.sign,
                self.job.pid,
                state_str,
                self.job.command,
                ampersand,
                width = STATE_COL_WIDTH
            )
        } else {
            write!(
                f,
                "[{}]{}  {:<width$} {}{}",
                self.job.id,
                self.sign,
                state_str,
                self.job.command,
                ampersand,
                width = STATE_COL_WIDTH
            )
        }
    }
}

pub fn check_background_jobs(jobs: &mut Vec<Job>, current: &mut usize, previous: &mut usize) {
    loop {
        let mut status = 0;

        // Check if a child job is terminated (`WNOHANG`) or paused (`WUNTRACED`).
        // It only waits if no options are passed here, i.e. 0 as the last argument.
        let pid = unsafe { c::waitpid(-1, &mut status, c::WNOHANG | c::WUNTRACED) };

        if pid <= 0 {
            return;
        }

        if let Some(index) = jobs.iter().position(|j| j.pid == pid) {
            // Case 1: Stopped (Ctrl+Z).
            if c::w_if_stopped(status) {
                let job = &mut jobs[index];
                job.state = State::Stopped;
                *previous = *current;
                *current = job.id;
                println!("[{}]+\tStopped\t\t{}", job.id, job.command);
            }
            // Case 2: Killed by signal (`kill` command or Ctrl+C).
            else if c::w_if_signaled(status) {
                let job = &jobs[index];
                println!("[{}]+\tTerminated\t{}", job.id, job.command);
                let removed_id = job.id;
                jobs.remove(index);
                if removed_id == *current {
                    *current = *previous;
                    *previous = 0;
                } else if removed_id == *previous {
                    *previous = 0;
                }
            }
            // Case 3: Finished of its own accord.
            else {
                let job = &jobs[index];
                let code = c::w_exitstatus(status);

                if code == 0 {
                    // Success.
                    println!("[{}]+\tDone\t\t{}", job.id, job.command);
                } else {
                    // Failure.
                    println!("[{}]+\tExit {}\t\t{}", job.id, code, job.command);
                }

                let removed_id = job.id;
                jobs.remove(index);
                if removed_id == *current {
                    *current = *previous;
                    *previous = 0;
                } else if removed_id == *previous {
                    *previous = 0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_form_pads_state_column_and_adds_ampersand() {
        let job = Job {
            id: 2,
            pid: 9999,
            command: "sleep 50".to_string(),
            state: State::Running,
        };

        let display = JobDisplay {
            job: &job,
            sign: "+",
            opts: JobOptions::default(),
        };

        let expected = format!(
            "[2]+  {:<width$} sleep 50 &",
            "Running",
            width = STATE_COL_WIDTH
        );

        assert_eq!(
            display.to_string(),
            expected,
            "short form running job should include state padding and ampersand"
        );
    }

    #[test]
    fn stopped_job_has_no_ampersand() {
        let job = Job {
            id: 2,
            pid: 9999,
            command: "sleep 50".to_string(),
            state: State::Stopped,
        };

        let display = JobDisplay {
            job: &job,
            sign: "+",
            opts: JobOptions::default(),
        };

        let expected = format!(
            "[2]+  {:<width$} sleep 50",
            "Stopped",
            width = STATE_COL_WIDTH
        );

        assert_eq!(
            display.to_string(),
            expected,
            "stopped job should include state padding but no ampersand"
        );
    }

    #[test]
    fn long_form_pads_state_and_pid_columns() {
        let job = Job {
            id: 1,
            pid: 8287,
            command: "ls ...".to_string(),
            state: State::Terminated,
        };

        let mut opts = JobOptions::default();
        opts.show_pid = true; // -l flag.

        let display = JobDisplay {
            job: &job,
            sign: "-",
            opts,
        };

        let expected = format!(
            "[1]- {:<5} {:<width$} ls ...",
            "8287",
            "Terminated",
            width = STATE_COL_WIDTH
        );

        assert_eq!(
            display.to_string(),
            expected,
            "long form should include pid column and correct padding"
        );
    }

    #[test]
    fn format_jobs_filters_by_running_only() {
        let jobs = vec![
            Job::new(1, 100, "run".into(), State::Running),
            Job::new(2, 101, "stop".into(), State::Stopped),
        ];

        let mut opts = JobOptions::default();
        opts.running_only = true; // -r

        let output = format_jobs(&jobs, opts, &[], 0, 0);

        assert!(
            output.contains("Running"),
            "output should contain running job when -r is used"
        );
        assert!(
            !output.contains("Stopped"),
            "output should not contain stopped job when -r is used"
        );
    }

    #[test]
    fn format_jobs_filters_by_specific_id() {
        let jobs = vec![
            Job::new(1, 100, "run".into(), State::Running),
            Job::new(2, 101, "stop".into(), State::Stopped),
        ];

        let opts = JobOptions::default();
        let filter = vec![2]; // Filter for Job ID 2 only.

        let output = format_jobs(&jobs, opts, &filter, 0, 0);

        assert!(
            !output.contains("run"),
            "output should not contain job id 1 when filtering for job id 2"
        );
        assert!(
            output.contains("stop"),
            "output should contain job id 2 when filtering for job id 2"
        );
    }

    #[test]
    fn format_jobs_marks_current_and_previous() {
        let jobs = vec![
            Job::new(1, 100, "first".into(), State::Running),
            Job::new(2, 101, "second".into(), State::Running),
            Job::new(3, 102, "third".into(), State::Running),
        ];

        let opts = JobOptions::default();
        let output = format_jobs(&jobs, opts, &[], 2, 1);

        assert!(
            output.contains("[2]+"),
            "job 2 should be marked current with '+'"
        );
        assert!(
            output.contains("[1]-"),
            "job 1 should be marked previous with '-'"
        );
        assert!(output.contains("[3] "), "job 3 should have no sign");
    }
}
