use std::sync::atomic::{AtomicI32, Ordering};

// Store the PID of the currently running foreground job.
// 0 means "no job running" (we're at the prompt).
pub static CURRENT_CHILD_PID: AtomicI32 = AtomicI32::new(0);

// At the start of `repl::repl`, `forward_handling`, below, is registered as the action associated with `Ctrl+C` and `Contrl+Z`; it will be called when either key is pressed. In the function, we forward the signal received by the shell to the current foreground child. The OS passes the signal number as the `sig` argument.

// WARNING: Be sure not to add `println!` or memory allocation (e.g., Box::new, String::new) inside a signal handler like this. That would risk deadlocking the program because these operations try to take a lock, but the signal might have interrupted the thread while it was already holding the same lock.

// WARNING: We mustn't put anything in this function that could panic! If we do, the Rust runtime won't know how to unwind the stack from here. It also runs the risk of deadlock if, say, it interrupts the main thread while the main thread is allocating memory. The memory allocator locks the head to find free space. For exammple, if the handler panics, it tries to acquire the lock on the heap to allocate memory to format the panic message and thus leads to deadlock.
pub extern "C" fn handle_forwarding(sig: i32) {
    let pid = CURRENT_CHILD_PID.load(Ordering::SeqCst);
    if pid > 0 {
        unsafe {
            libc::kill(pid, sig);
        }
    }
}
