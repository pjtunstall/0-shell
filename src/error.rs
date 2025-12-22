use crate::ansi::{BOLD, ERROR_COLOR, RESET_FG};

pub fn handle_error(command: &str, err: String) {
    if err.starts_with("0-shell: ") {
        red_println(&format!("{}", err));
    } else {
        red_println(&format!("{}: {}", command, err));
    }
}

pub fn red_println(text: &str) {
    // Reset only the foreground color so we keep any active bold styling.
    eprintln!("{ERROR_COLOR}{BOLD}{text}{RESET_FG}", text = text);
}

// Emit a minimal, async-signal-safe error message after execvp failure. POSIX
// guarantees write(2) is async-signal-safe, so calling `libc::write` in a
// signal handler or post-fork child is allowed. `println!` goes through Rust’s
// buffered stdout, which takes locks and can allocate, so it's not
// async-signal-safe.
pub fn print_exec_failure(cmd_bytes: &[u8]) {
    let prefix = b"0-shell: command not found: ";
    unsafe {
        let _ = libc::write(
            libc::STDERR_FILENO,
            ERROR_COLOR.as_bytes().as_ptr() as *const libc::c_void,
            ERROR_COLOR.len(),
        );
        let _ = libc::write(
            libc::STDERR_FILENO,
            BOLD.as_bytes().as_ptr() as *const libc::c_void,
            BOLD.len(),
        );
        let _ = libc::write(
            libc::STDERR_FILENO,
            prefix.as_ptr() as *const libc::c_void,
            prefix.len(),
        );
        let _ = libc::write(
            libc::STDERR_FILENO,
            cmd_bytes.as_ptr() as *const libc::c_void,
            cmd_bytes.len(),
        );
        let _ = libc::write(
            libc::STDERR_FILENO,
            RESET_FG.as_bytes().as_ptr() as *const libc::c_void,
            RESET_FG.len(),
        );
        let _ = libc::write(
            libc::STDERR_FILENO,
            b"\n".as_ptr() as *const libc::c_void,
            1,
        );
    }
}
