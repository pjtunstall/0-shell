use std::sync::atomic::{AtomicI32, Ordering};

// Flags.
pub const WNOHANG: i32 = 1;
pub const WUNTRACED: i32 = 2;

// Signals.
pub const SIGINT: i32 = 2;
pub const SIGKILL: i32 = 9;
pub const SIGTERM: i32 = 15;
pub const SIGCONT: i32 = 18;
pub const SIGSTOP: i32 = 19;
pub const SIGTSTP: i32 = 20;

// Constants for `tcsetattr`.
pub const STDIN_FILENO: i32 = 0;
pub const TCSANOW: i32 = 0;

// Store the PID of the currently running foreground job.
// 0 means "no job running" (we're at the prompt).
pub static CURRENT_CHILD_PID: AtomicI32 = AtomicI32::new(0);

// The termios struct (standard x86_64 layout)
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Termios {
    pub c_iflag: u32,
    pub c_oflag: u32,
    pub c_cflag: u32,
    pub c_lflag: u32,
    pub c_line: u8,
    pub c_cc: [u8; 32],
    pub c_ispeed: u32,
    pub c_ospeed: u32,
}

unsafe extern "C" {
    pub fn signal(sig: i32, handler: extern "C" fn(i32)) -> usize;
    pub fn kill(pid: i32, sig: i32) -> i32;
    pub fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32;

    // Terminal attribute functions.
    pub fn tcgetattr(fd: i32, termptr: *mut Termios) -> i32;
    pub fn tcsetattr(fd: i32, optional_actions: i32, termptr: *const Termios) -> i32;

    // Group management.
    pub fn setpgid(pid: i32, pgid: i32) -> i32;
    pub fn tcsetpgrp(fd: i32, pgrp: i32) -> i32;
    pub fn getpgrp() -> i32;
    pub fn getpid() -> i32;
}

// Forward the signal received by the shell to the current foreground child.
// The OS passes the signal number (e.g., 2 or 20) as the `sig` argument.
pub extern "C" fn handle_forwarding(sig: i32) {
    let pid = CURRENT_CHILD_PID.load(Ordering::SeqCst);
    if pid > 0 {
        unsafe {
            kill(pid, sig);
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
