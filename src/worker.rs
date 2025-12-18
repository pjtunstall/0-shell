use crate::c;
use crate::commands::jobs::{Job, State};
use std::{env, process::Command, sync::atomic::Ordering};

pub fn launch_job(
    args: &[String],
    jobs: &mut Vec<Job>,
    is_background: bool,
) -> Result<String, String> {
    let self_path = env::current_exe().map_err(|e| format!("Unable to get own path: {}", e))?;

    let child = Command::new(self_path)
        .arg("--internal-worker")
        .args(args)
        .spawn()
        .expect("failed to spawn");

    let pid = child.id() as i32;

    if is_background {
        let id = jobs.len() + 1;
        let command_string = args.join(" ");
        jobs.push(Job::new(id, pid, command_string, State::Running));
        let output = format!("[{}] {}\n", id, pid);
        return Ok(output);
    }

    // Register this PID as target for Ctrl+C.
    c::CURRENT_CHILD_PID.store(pid, Ordering::Relaxed);

    // Use raw `waitpid`` to detect "Stopped" state.
    let mut status = 0;
    unsafe {
        // Blocks until the child process either dies or stops (`WUNTRACED`).
        c::waitpid(pid, &mut status, c::WUNTRACED);
    }

    // Unregister: the foreground slot is now free (either dead or moved to bg).
    c::CURRENT_CHILD_PID.store(0, Ordering::Relaxed);

    if c::w_if_stopped(status) {
        // Case A: Ctrl+Z.
        let id = jobs.len() + 1;
        let command_string = args.join(" ");

        let new_job = Job::new(id, pid, command_string.clone(), State::Stopped);
        jobs.push(new_job);

        println!("\n[{}]+\tStopped\t\t{}", id, command_string);

        Ok(String::new())
    } else {
        // CASE B: normal exit or Ctrl+C.
        // We could check `WEXITSTATUS` here to see if the command failed (exit code > 0), but for now, we just return control to the repl.
        Ok(String::new())
    }
}
