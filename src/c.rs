use std::sync::atomic::{AtomicI32, Ordering};

pub const WUNTRACED: i32 = 2;
pub const SIGINT: i32 = 2;
pub const SIGTSTP: i32 = 20;
pub const SIGCONT: i32 = 18;

// Store the PID of the currently running foreground job.
// 0 means "no job running" (we're at the prompt).
pub static CURRENT_CHILD_PID: AtomicI32 = AtomicI32::new(0);

unsafe extern "C" {
    pub fn signal(sig: i32, handler: extern "C" fn(i32)) -> usize;
    pub fn kill(pid: i32, sig: i32) -> i32;
    pub fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32;
}

// Forward the signal received by the shell to the current foreground child.
// The OS passes the signal number (e.g., 2 or 20) as the `sig` argument.
pub extern "C" fn handle_forwarding(sig: i32) {
    let pid = CURRENT_CHILD_PID.load(Ordering::Relaxed);
    if pid > 0 {
        unsafe {
            kill(pid, sig);
        }
    }
}

// Check if the process was suspended (Ctrl+Z).
// Bitwise logic: in Linux, a stopped process has 0x7F in the lower 8 bits.
pub fn w_if_stopped(status: i32) -> bool {
    (status & 0xff) == 0x7f
}
