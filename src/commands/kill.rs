use crate::{
    c::{self, SIGTERM},
    commands::jobs::{self, Job},
};

pub const USAGE: &str = "Usage: kill PID";

pub fn kill(input: &[String], jobs: &mut Vec<Job>) -> Result<String, String> {
    jobs::check_background_jobs(jobs);

    if input.len() > 2 {
        return Err(format!("Too many arguments\n{}", USAGE));
    }

    if input.len() < 2 {
        return Err(format!("Not enough arguments\n{}", USAGE));
    }

    let pid: i32 = input[1]
        .parse()
        .map_err(|e| format!("Failed to parse PID: {}\n{}", e, input.last().unwrap()))?;
    if pid < 0 {
        return Err(format!("PID must be positive"));
    }
    if pid == 0 {
        return Err(format!(""));
    }

    // As a safety measure, I've chosen to only let it kill jobs in the jobs list. Remove this check to allow it to kill any process.
    let job_exists = jobs.iter().any(|j| j.pid == pid);
    if !job_exists {
        return Err(format!("No job with PID: {}", pid));
    }

    unsafe {
        c::kill(pid, SIGTERM);
    }

    Ok(String::new())
}
