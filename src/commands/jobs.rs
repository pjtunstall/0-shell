use std::borrow::Borrow;

use crate::c;

pub const USAGE: &str = "Usage: jobs [-l]";
const STATE_COL_WIDTH: usize = 24;

pub fn jobs(input: &[String], jobs: &mut Vec<Job>) -> Result<String, String> {
    check_background_jobs(jobs);

    if input.len() > 2 {
        return Err(format!("Too many arguments\n{}", USAGE));
    }

    let is_long = if input.len() == 2 {
        if input[1] == "-l" {
            true
        } else {
            return Err(format!("Unknown option\n{}", USAGE));
        }
    } else {
        false
    };

    let output = format_jobs(jobs, is_long);
    Ok(output)
}

pub fn format_jobs<T: Borrow<Job>>(items: &[T], is_long: bool) -> String {
    let mut output = String::new();

    for (i, item) in items.iter().enumerate() {
        let job = item.borrow();

        let sign = if i == items.len() - 1 {
            "+"
        } else if i == items.len().saturating_sub(2) {
            "-"
        } else {
            " "
        };

        let display = JobDisplay { job, sign, is_long };

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
            State::Terminated => "Terminated",
        };
        f.write_str(s)
    }
}

pub struct JobDisplay<'a> {
    pub job: &'a Job,
    pub sign: &'a str,
    pub is_long: bool,
}

impl std::fmt::Display for JobDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state_str = self.job.state.to_string();

        if self.is_long {
            write!(
                f,
                "[{}]{} {:<5} {:<width$} {}",
                self.job.id,
                self.sign,
                self.job.pid,
                state_str,
                self.job.command,
                width = STATE_COL_WIDTH
            )
        } else {
            write!(
                f,
                "[{}]{} {:<width$} {}",
                self.job.id,
                self.sign,
                state_str,
                self.job.command,
                width = STATE_COL_WIDTH
            )
        }
    }
}

pub fn check_background_jobs(jobs: &mut Vec<Job>) {
    loop {
        let mut status = 0;

        // Check if a child job is terminated (`WNOHANG`). It only waits if no options are passed here, i.e. `0` in the options slot; we don't want to call this from the parent or 0-shell will freeze.
        // -1: is a wildcard; it means check for any pid.
        // 0: any child in the same process group as the shell.
        // < -1: for any child in the group number with the number specified.
        let dead_pid = unsafe { c::waitpid(-1, &mut status, c::WNOHANG) };

        // dead_pid > 0: We found a terminated job.
        // dead_pid == 0: None found. They're all running fine.
        // dead_pid == -1: Error. Usually means no children exist.
        if dead_pid <= 0 {
            return; // Stop looking. Return to the prompt.
        }

        // We found a dead job: update the list.
        if let Some(index) = jobs.iter().position(|j| j.pid == dead_pid) {
            let job = &jobs[index];

            // TODO: check `status` to see if they segfaulted?
            println!("[{}]+\tDone\t\t{}", job.id, job.command);

            jobs.remove(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_form_pads_state_column() {
        let job = Job {
            id: 2,
            pid: 9999,
            command: "sleep 50 &".to_string(),
            state: State::Running,
        };

        let display = JobDisplay {
            job: &job,
            sign: "+",
            is_long: false,
        };

        let expected = format!(
            "[2]+ {:<width$} sleep 50 &",
            "Running",
            width = STATE_COL_WIDTH
        );

        assert_eq!(display.to_string(), expected);
    }

    #[test]
    fn long_form_pads_state_and_pid_columns() {
        let job = Job {
            id: 1,
            pid: 8287,
            command: "ls ...".to_string(),
            state: State::Terminated,
        };

        let display = JobDisplay {
            job: &job,
            sign: "-",
            is_long: true,
        };

        let expected = format!(
            "[1]- {:<5} {:<width$} ls ...",
            "8287",
            "Terminated",
            width = STATE_COL_WIDTH
        );

        assert_eq!(display.to_string(), expected);
    }
}
