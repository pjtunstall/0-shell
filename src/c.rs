use std::sync::atomic::{AtomicI32, Ordering};

pub use libc::termios as Termios;
pub use libc::{self, *};

// Store the PID of the currently running foreground job.
// 0 means "no job running" (we're at the prompt).
pub static CURRENT_CHILD_PID: AtomicI32 = AtomicI32::new(0);

// Forward the signal received by the shell to the current foreground child.
// The OS passes the signal number (e.g., 2 or 20) as the `sig` argument.
pub extern "C" fn handle_forwarding(sig: i32) {
    let pid = CURRENT_CHILD_PID.load(Ordering::SeqCst);
    if pid > 0 {
        unsafe {
            libc::kill(pid, sig);
        }
    }
}

// A stopped process has 0x7f in the lower 8 bits.
pub fn w_if_stopped(status: i32) -> bool {
    (status & 0xff) == 0x7f
}

// Check if killed by a signal.
pub fn w_if_signaled(status: i32) -> bool {
    // If low 7 bits are 0, it exited normally (WIFEXITED).
    // If low 7 bits are 0x7f, it is stopped (WIFSTOPPED).
    // Anything in between (1..126) is a signal (WIFSIGNALED).
    let term_sig = status & 0x7f;
    term_sig > 0 && term_sig < 0x7f
}

// The exit code lives in the high byte (bits 8-15). Shift it down to read it.
pub fn w_exitstatus(status: i32) -> i32 {
    (status >> 8) & 0xff
}

pub fn w_if_exited(status: i32) -> bool {
    (status & 0x7f) == 0
}

pub fn w_if_continued(status: i32) -> bool {
    status == 0xffff
}
