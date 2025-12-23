use std::ffi::CString;

use crate::error;

pub fn run_child(ptrs: Vec<*const i8>, c_strings: Vec<CString>) {
    unsafe {
        let child_pid = libc::getpid(); // The child learns its own PID.

        // Make the child the leader of a new process group. We place this line
        // in both parent and child code because we don't know which the OS will
        // choose to run first.
        libc::setpgid(child_pid, child_pid);

        restore_default_signal_handlers(); // Stop ignoring signals.

        // Replace the current process image with the target program, keeping
        // the same PID. (V = vector args; P = path lookup.)
        libc::execvp(ptrs[0], ptrs.as_ptr());

        // If we reach this line, `execvp` failed.
        error::print_exec_failure(c_strings[0].as_bytes());
        std::process::exit(1);
    }
}

fn restore_default_signal_handlers() {
    unsafe {
        libc::signal(libc::SIGINT, libc::SIG_DFL); // Ctrl+C
        libc::signal(libc::SIGTSTP, libc::SIG_DFL); // Ctrl+Z
        libc::signal(libc::SIGQUIT, libc::SIG_DFL); // Ctrl+\

        // Allow SIGTTIN to suspend us if we try to read from the TTY while in
        // the background.
        libc::signal(libc::SIGTTIN, libc::SIG_DFL);

        // Allow SIGTTOU to suspend us if we try to write to the TTY or take
        // control of the terminal while in the background.
        libc::signal(libc::SIGTTOU, libc::SIG_DFL);
    }
}
