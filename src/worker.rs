use crate::c;
use crate::commands::jobs::Job;
use std::{env, process::Command, sync::atomic::Ordering};

pub fn launch_worker_process(args: &[String], jobs: &mut Vec<Job>) -> Result<String, String> {
    let self_path = env::current_exe().map_err(|e| format!("Unable to get own path: {}", e))?;

    let child = Command::new(self_path)
        .arg("--internal-worker")
        .args(args)
        .spawn()
        .expect("failed to spawn");

    let pid = child.id() as i32;

    // Register this PID as target for Ctrl+C.
    c::CURRENT_CHILD_PID.store(pid, Ordering::Relaxed);

    // Use raw `waitpid`` to detect "Stopped" state.
    let mut status = 0;
    unsafe {
        // Blocks until the child either dies or stops (`WUNTRACED`).
        c::waitpid(pid, &mut status, c::WUNTRACED);
    }

    // Unregister: the foreground slot is now free (either dead or moved to bg).
    c::CURRENT_CHILD_PID.store(0, Ordering::Relaxed);

    if c::w_if_stopped(status) {
        // Case A: Ctrl+Z.
        let id = jobs.len() + 1;
        let command_string = args.join(" ");

        let new_job = Job::new(id, pid, command_string.clone());
        jobs.push(new_job);

        println!("\n[{}]+\tStopped\t\t{}", id, command_string);

        Ok(String::new())
    } else {
        // CASE B: normal exit or Ctrl+C.
        // We could check `WEXITSTATUS` here to see if the command failed (exit code > 0), but for now, we just return control to the repl.
        Ok(String::new())
    }
}
