use crate::commands::jobs::{Job, State};

pub const USAGE: &str = "Usage:\texit [status]";
pub const STOPPED_JOBS_WARNING: &str = "There are stopped jobs.";

pub fn exit(args: &[String], jobs: &[Job], exit_attempted: &mut bool) -> Result<String, String> {
    let has_stopped = jobs.iter().any(|j| j.state == State::Stopped);

    if has_stopped && !*exit_attempted {
        *exit_attempted = true;
        return Err(STOPPED_JOBS_WARNING.to_string());
    }

    if args.len() > 2 {
        return Err(format!("Too many arguments\n{}", USAGE));
    }

    let mut exit_code = 0;
    if args.len() == 2 {
        exit_code = args[1]
            .parse::<i32>()
            .map_err(|_| format!("Numeric argument required: {}", args[1]))?;
    }

    Ok(exit_code.to_string())
}
