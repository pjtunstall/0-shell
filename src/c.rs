use std::sync::atomic::{AtomicI32, Ordering};

pub const SIGINT: i32 = 2;

// Stores the PID of the currently running foreground job.
// 0 means "no job running" (we're at the prompt).
pub static CURRENT_CHILD_PID: AtomicI32 = AtomicI32::new(0);

unsafe extern "C" {
    // Register this event handler with the OS: if we receive signal `sig` (we'll pass `SIGINT`), run `handler` instead of the default action (in this case, the default is to kill the current process). We'll pass `handle_sigint` as the `handler`.
    // Aside: the C function corresponding to `signal` returns a function pointer to the previous handler (so you can restore it later if you want). In Rust FFI, representing a function pointer as a usize (an integer the same size as a pointer) is a standard, convenient way to treat that address when you don't intend to call it immediately. It captures the address safely.
    pub fn signal(sig: i32, handler: extern "C" fn(i32)) -> usize;

    // Forward the signal `sig` to the child process with ID `pid`.
    pub fn kill(pid: i32, sig: i32) -> i32;
}

pub extern "C" fn handle_sigint(_signal: i32) {
    let pid = CURRENT_CHILD_PID.load(Ordering::Relaxed);
    if pid > 0 {
        unsafe {
            // Forward the signal `SIGINT`  to the process with ID `pid`.
            // `kill` is bit of a misnomer. It really means forward the given signal. Some signals do kills the process, but not all.
            kill(pid, SIGINT);
        }
    }
}
