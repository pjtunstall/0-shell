use std::sync::atomic::{AtomicI32, Ordering};

// Store the PID of the currently running foreground job.
// 0 means "no job running" (we're at the prompt).
pub static CURRENT_CHILD_PID: AtomicI32 = AtomicI32::new(0);

// Forward the signal received by the shell to the current foreground child.
// The OS passes the signal number (e.g., 2 or 20) as the `sig` argument.
// WARNING: Be sure not to add `println!` or memory allocation (e.g., Box::new, String::new) inside this function, and indeeed all non-async-signal-safe functions. That would risk deadlocking the program because these operations try to take a lock, but the signal might have interrupted the thread while it was already holding the same lock.
pub extern "C" fn handle_forwarding(sig: i32) {
    let pid = CURRENT_CHILD_PID.load(Ordering::SeqCst); // Ok to read because atomic.
    if pid > 0 {
        unsafe {
            libc::kill(pid, sig);
        }
    }
}
