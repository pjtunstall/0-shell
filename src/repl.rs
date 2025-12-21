use std::{collections::VecDeque, io, mem, ptr};

use crate::{
    ansi::{BOLD, RESET},
    c,
    commands::{
        self,
        exit::STOPPED_JOBS_WARNING,
        jobs::{self, Job},
    },
    error, input,
};

struct TextStyle;

impl TextStyle {
    fn new() -> Self {
        print!("{BOLD}");
        TextStyle
    }
}

impl Drop for TextStyle {
    fn drop(&mut self) {
        print!("{RESET}");
    }
}

// When this struct is instantiated at the start of `repl`, it saves the terminal attributes so that it can restore them on drop to prevent lingering no-echo/cbreak states if an interactive child leaves the TTY altered. The non-Unix version below is a no-op placeholder.
struct TtyGuard {
    saved: Option<libc::termios>,
}

impl TtyGuard {
    fn new() -> Self {
        unsafe {
            let mut tio = std::mem::zeroed::<libc::termios>();
            if libc::tcgetattr(libc::STDIN_FILENO, &mut tio) == 0 {
                Self { saved: Some(tio) }
            } else {
                Self { saved: None }
            }
        }
    }
}

impl Drop for TtyGuard {
    fn drop(&mut self) {
        if let Some(saved) = self.saved {
            unsafe {
                libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &saved);
            }
        }
    }
}

pub fn repl() {
    let mut jobs: Vec<Job> = Vec::new();
    let mut current: usize = 0;
    let mut previous: usize = 0;
    let mut exit_attempted = false;
    let mut final_status = 0;

    unsafe {
        // Declare `forward` as a C "signal action" struct, and zero its memory. This struct represents an action: a callback function and some configuration. Below, two calls to the Rust `set_action` closure will register `forward` as the action associated with `SIGINT` and `SIGTSTP` respectively.
        let mut forward = mem::zeroed::<libc::sigaction>();

        // The `sa_mask` field is a bitmask indicating which other signals to block while the current signal is being handled. This line says block them all. It ensures that no other signal can interrupt the current action. They're queued till it's finished.
        libc::sigfillset(&mut forward.sa_mask);

        // After running the signal handler, automatically restart any interrupted I/O syscall instead of the default behavior (which would be to fail with an `EINTR` error).
        forward.sa_flags = libc::SA_RESTART;

        // Set this action's callback function to `c::handle_forwarding` (a function pointer cast to `usize`), that will forward the signal to the process whose PID is stored in `static CURRENT_CHILD_PID: AtomicI32` (defined in `crate::c`).
        forward.sa_sigaction = c::handle_forwarding as usize;

        // Analogous to `forward`, we define an `ignore` action. Its callback, `libc::SIG_IGN`, tells the kernel to ignore whatever signal it's registered with.
        let mut ignore = mem::zeroed::<libc::sigaction>();
        libc::sigemptyset(&mut ignore.sa_mask);
        ignore.sa_sigaction = libc::SIG_IGN;

        // Define a closure to register a signal handler if it can and panic if not.
        let set_action = |sig: i32, act: &libc::sigaction| {
            // `null_mut`: This argument is for the "old action". If we passed a pointer here, the OS would write the previous signal handler settings into it. Since we don't care what the previous handler was, we pass a null pointer.
            if libc::sigaction(sig, act, ptr::null_mut()) != 0 {
                let err = io::Error::last_os_error();
                panic!("failed to set signal action `{}`: {}", sig, err);
            }
        };

        set_action(libc::SIGINT, &forward); // Ctr+C: terminate.
        set_action(libc::SIGTSTP, &forward); // Ctrl+Z: stop (i.e. pause).

        // Prevent the shell from being stopped if it attempts a terminal read or write while backgrounded. The terminal driver fires `SIGTTIN` (`SIGTTOU`) when a background process group tries to read from the terminal, or write to it (unless output is piped or redirected).
        set_action(libc::SIGTTIN, &ignore); // Ignore input (except control chars).
        set_action(libc::SIGTTOU, &ignore); // Ignore output.
    }

    let mut history = VecDeque::new();
    history.push_back(String::new());

    {
        let _style = TextStyle::new();
        let _tty_guard = TtyGuard::new();

        loop {
            jobs::check_background_jobs(&mut jobs, &mut current, &mut previous);

            let input_string = match input::get_input(&mut history) {
                Ok(Some(ok_input)) => ok_input,
                Ok(None) => {
                    // Ctrl+D.
                    let has_stopped = jobs.iter().any(|j| j.state == jobs::State::Stopped);
                    if has_stopped && !exit_attempted {
                        error::red_println(&format!("exit: {}", STOPPED_JOBS_WARNING));
                        exit_attempted = true;
                        continue;
                    }
                    break;
                }
                Err(err) => {
                    error::red_println(&format!("0-shell: Failed to get input: {}", err));
                    continue;
                }
            };

            if input_string.trim().is_empty() {
                continue;
            }

            history.push_back(input_string.clone());

            let input_after_splitting: Vec<String>;
            match input::split::split(&input_string) {
                Ok(res) => input_after_splitting = res,
                Err(err) => {
                    error::red_println(&format!("0-shell: {}", &err));
                    continue;
                }
            }

            if input_after_splitting.is_empty() {
                continue;
            }

            let (exit_signal, result) = commands::run_command(
                &input_after_splitting,
                &mut jobs,
                &mut current,
                &mut previous,
                &mut exit_attempted,
            );

            match result {
                Ok(ok) => {
                    if !ok.is_empty() {
                        print!("{}", ok);
                    }
                }
                Err(err) => error::handle_error(&input_after_splitting[0], err),
            }

            if let Some(code) = exit_signal {
                final_status = code;
                break;
            }
        }
    }

    for job in jobs {
        unsafe {
            libc::kill(-job.pid, libc::SIGHUP);
            libc::kill(-job.pid, libc::SIGCONT);
        }
    }

    std::process::exit(final_status);
}
