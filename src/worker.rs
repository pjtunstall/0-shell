use std::{env, process::Command, sync::atomic::Ordering};

use crate::c;

pub fn launch_worker_process(args: &[String]) -> Result<String, String> {
    let self_path = env::current_exe().map_err(|e| format!("Unable to get own path: {}", e))?;

    let mut child = Command::new(self_path)
        .arg("--internal-worker")
        .args(args)
        .spawn()
        .expect("failed to spawn");

    // Add worker PID to store so that we can kill it. The child received a coopy of our memory at when we launched it, but it doesn't see us adding its id later here.
    c::CURRENT_CHILD_PID.store(child.id() as i32, Ordering::Relaxed);

    // println!("Launched job with PID: {}", child.id());

    let status = child.wait();

    // Unregister child: stop targeting this PID.
    c::CURRENT_CHILD_PID.store(0, Ordering::Relaxed);

    let s = status.map_err(|e| format!("wait failed: {}", e))?;
    if s.success() || s.code().is_none() {
        // The code is `None` if the worker is terminated by SIGINT.
        Ok(String::new())
    } else {
        Err(format!("process terminated: {}", s))
    }
}
