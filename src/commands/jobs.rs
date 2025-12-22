use std::borrow::Borrow;

use libc;

pub const USAGE: &str = "Usage:\tjobs [-lprs] [jobspec ...]";
pub const OPTIONS_USAGE: &str = "\r\n-l      -- show process IDs\r\n-p      -- show only process IDs\r\n-r      -- show only running jobs\r\n-s      -- show only stopped jobs";
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

pub struct Job {
    pub jid: usize, // Job ID: 1, 2, 3, ...
    pub pgid: i32,  // Process group ID: the PID of the group leader.
    pub command: String,
    pub state: State,
}

impl Job {
    pub fn new(jid: usize, pgid: i32, command: String, state: State) -> Self {
        assert!(pgid > 0, "`pgid` must be positive");
        Self {
            jid,
            pgid,
            command,
            state,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum State {
    Running,
    Stopped,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            State::Running => "Running",
            State::Stopped => "Stopped",
        };
        f.write_str(s)
    }
}

pub struct JobDisplay<'a> {
    pub job: &'a Job,
    pub sign: &'a str,
    pub opts: JobOptions,
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
                    _ => return Err(format!("Invalid option -- '{c}'\n{USAGE}")),
                }
            }
        } else {
            let id = resolve_jobspec(arg, *current, *previous)?;
            specific_job_ids.push(id);
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

        // Filter by specific Job IDs (if user asked for specific jobs).
        if !filter_ids.is_empty() && !filter_ids.contains(&job.jid) {
            continue;
        }

        // Filter: -r (Running only).
        if opts.running_only && matches!(job.state, State::Stopped) {
            continue;
        }
        // Filter: -s (Stopped only).
        if opts.stopped_only && matches!(job.state, State::Running) {
            continue;
        }

        // Mode: -p (PIDs only).
        if opts.pid_only {
            let pgid = job.pgid;
            output.push_str(&format!("{pgid}\n"));
            continue;
        }

        let sign = if job.jid == current {
            "+"
        } else if job.jid == previous {
            "-"
        } else {
            " "
        };

        let display = JobDisplay { job, sign, opts };
        output.push_str(&format!("{display}\n"));
    }

    output
}

pub fn resolve_jobspec(spec: &str, current: usize, previous: usize) -> Result<usize, String> {
    let raw = if spec == "%" || spec == "%+" || spec == "%%" {
        if current > 0 {
            return Ok(current);
        } else if previous > 0 {
            // No explicit current; fall back to previous.
            return Ok(previous);
        } else {
            return Err(String::from("Current: no such job"));
        }
    } else if spec == "%-" {
        if previous > 0 {
            return Ok(previous);
        } else if current > 0 {
            // Single-job case: %- maps to current.
            return Ok(current);
        } else {
            return Err(String::from("Current: no such job"));
        }
    } else if let Some(rest) = spec.strip_prefix('%') {
        rest
    } else {
        spec
    };

    raw.parse::<usize>()
        .map_err(|_| format!("Invalid job ID: {spec}"))
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
                self.job.jid,
                self.sign,
                self.job.pgid,
                state_str,
                self.job.command,
                ampersand,
                width = STATE_COL_WIDTH
            )
        } else {
            write!(
                f,
                "[{}]{}  {:<width$} {}{}",
                self.job.jid,
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

        // WNOHANG: Poll once but don't wait. WUNTRACED: Report stopped
        // processes too. (By default `waitpid` only reports terminated
        // processes.)
        let pid = unsafe {
            libc::waitpid(
                -1,
                &mut status,
                libc::WNOHANG | libc::WUNTRACED | libc::WCONTINUED,
            )
        };

        if pid <= 0 {
            break; // No more children have changed state.
        }

        if let Some(index) = jobs.iter().position(|j| j.pgid == pid) {
            if libc::WIFSTOPPED(status) {
                // The kernel stopped the process (e.g. Python background input).
                if jobs[index].state != State::Stopped {
                    jobs[index].state = State::Stopped;
                    *previous = *current;
                    *current = jobs[index].jid;
                    println!(
                        "\n[{}]+\tStopped\t\t{}",
                        jobs[index].jid, jobs[index].command
                    );
                }
            } else if libc::WIFEXITED(status) || libc::WIFSIGNALED(status) {
                // The process is dead.
                let job = &jobs[index];

                if libc::WIFSIGNALED(status) {
                    println!("[{}]+\tTerminated\t{}", job.jid, job.command);
                } else {
                    let code = libc::WEXITSTATUS(status);
                    if code == 0 {
                        println!("[{}]+\tDone\t\t{}", job.jid, job.command);
                    } else {
                        println!("[{}]+\tExit {}\t\t{}", job.jid, code, job.command);
                    }
                }

                let removed_id = job.jid;
                jobs.remove(index);

                // Update pointers if the removed job was the current or
                // previous one.
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
            jid: 2,
            pgid: 9999,
            command: String::from("sleep 50"),
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
            jid: 2,
            pgid: 9999,
            command: String::from("sleep 50"),
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
            jid: 1,
            pgid: 8287,
            command: String::from("ls ..."),
            state: State::Stopped,
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
            "Stopped",
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
            Job::new(1, 100, String::from("run"), State::Running),
            Job::new(2, 101, String::from("stop"), State::Stopped),
        ];

        let mut opts = JobOptions::default();
        opts.running_only = true; // -r

        let output = format_jobs(&jobs, opts, &[], 0, 0);

        assert!(
            output.contains("Running"),
            "output should contain running job when `-r` is used"
        );
        assert!(
            !output.contains("Stopped"),
            "output should not contain stopped job when `-r` is used"
        );
    }

    #[test]
    fn format_jobs_filters_by_specific_id() {
        let jobs = vec![
            Job::new(1, 100, String::from("run"), State::Running),
            Job::new(2, 101, String::from("stop"), State::Stopped),
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
            Job::new(1, 100, String::from("first"), State::Running),
            Job::new(2, 101, String::from("second"), State::Running),
            Job::new(3, 102, String::from("third"), State::Running),
        ];

        let opts = JobOptions::default();
        let output = format_jobs(&jobs, opts, &[], 2, 1);

        assert!(
            output.contains("[2]+"),
            "job 2 should be marked current with `+`"
        );
        assert!(
            output.contains("[1]-"),
            "job 1 should be marked previous with `-`"
        );
        assert!(output.contains("[3] "), "job 3 should have no sign");
    }

    #[test]
    fn resolve_jobspec_handles_current_and_previous_aliases() {
        assert_eq!(
            resolve_jobspec("%+", 2, 1).unwrap(),
            2,
            "%+ should resolve to current job"
        );
        assert_eq!(
            resolve_jobspec("%%", 2, 1).unwrap(),
            2,
            "%% should resolve to current job"
        );
        assert_eq!(
            resolve_jobspec("%", 2, 1).unwrap(),
            2,
            "% should resolve to current job"
        );
        assert_eq!(
            resolve_jobspec("%-", 2, 1).unwrap(),
            1,
            "%- should resolve to previous job"
        );

        // Single-job case: %- falls back to current.
        assert_eq!(
            resolve_jobspec("%-", 3, 0).unwrap(),
            3,
            "%- should fall back to current when previous is 0"
        );
    }
}
