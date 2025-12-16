const USAGE: &str = "Usage: jobs [-l]";
const STATE_COL_WIDTH: usize = 24;

pub fn jobs(input: &[String], jobs: &[Job]) -> Result<String, String> {
    if input.len() > 2 {
        return Err(format!("too many arguments\n{}", USAGE));
    }

    let is_long = if input.len() == 2 {
        if input[1] == "-l" {
            true
        } else {
            return Err(format!("unknown option\n{}", USAGE));
        }
    } else {
        false
    };

    let mut output = String::new();

    for (i, job) in jobs.iter().enumerate() {
        let sign = if i == jobs.len() - 1 {
            "+"
        } else if i == jobs.len().saturating_sub(2) {
            "-"
        } else {
            " "
        };

        let display = JobDisplay { job, sign, is_long };

        output.push_str(&format!("{}\n", display));
    }

    Ok(output)
}

pub struct Job {
    pub id: usize,
    pub pid: i32,
    pub command: String,
    pub state: State,
}

impl Job {
    pub fn new(jobs_total: usize, pid: i32, command: String) -> Self {
        assert!(pid > 0, "`pid` must be positive");

        Self {
            id: jobs_total,
            pid,
            command,
            state: State::Stopped,
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

struct JobDisplay<'a> {
    job: &'a Job,
    sign: &'a str,
    is_long: bool,
}

impl std::fmt::Display for JobDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state_str = self.job.state.to_string();

        if self.is_long {
            write!(
                f,
                "[{}] {} {:<5} {:<width$} {}",
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
                "[{}] {} {:<width$} {}",
                self.job.id,
                self.sign,
                state_str,
                self.job.command,
                width = STATE_COL_WIDTH
            )
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
            "[2] + {:<width$} sleep 50 &",
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
            "[1] - {:<5} {:<width$} ls ...",
            "8287",
            "Terminated",
            width = STATE_COL_WIDTH
        );

        assert_eq!(display.to_string(), expected);
    }
}
