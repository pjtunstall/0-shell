use std::collections::VecDeque;

use crate::{
    c::{self, *},
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
        print!("\x1b[1m");
        TextStyle
    }
}

impl Drop for TextStyle {
    fn drop(&mut self) {
        print!("\x1b[0m");
    }
}

// When this struct is instantiated at the start of `repl`, it saves the terminal attributes so that it can restore them on drop to prevent lingering no-echo/cbreak states if an interactive child leaves the TTY altered. The non-Unix version below is a no-op placeholder.
#[cfg(unix)]
struct TtyGuard {
    saved: Option<c::Termios>,
}

#[cfg(unix)]
impl TtyGuard {
    fn new() -> Self {
        unsafe {
            let mut tio = std::mem::zeroed::<c::Termios>();
            if c::tcgetattr(STDIN_FILENO, &mut tio) == 0 {
                Self { saved: Some(tio) }
            } else {
                Self { saved: None }
            }
        }
    }
}

#[cfg(unix)]
impl Drop for TtyGuard {
    fn drop(&mut self) {
        if let Some(saved) = self.saved {
            unsafe {
                c::tcsetattr(STDIN_FILENO, TCSANOW, &saved);
            }
        }
    }
}

// For non-Unix-like operating systems, a non-functional placeholder.
// Expect a struggle between background and foreground processes if
// they both try to interact with the terminal, e.g. 0-shell in the
// foreground and Python in the background, as in one of the
// job-control audit questions.
#[cfg(not(unix))]
struct TtyGuard;
#[cfg(not(unix))]
impl TtyGuard {
    fn new() -> Self {
        TtyGuard
    }
}
#[cfg(not(unix))]
impl Drop for TtyGuard {
    fn drop(&mut self) {}
}

pub fn repl() {
    let mut jobs: Vec<Job> = Vec::new();
    let mut current: usize = 0;
    let mut previous: usize = 0;
    let mut exit_attempted = false;
    let mut final_status = 0;

    unsafe {
        let handler_ptr = c::handle_forwarding as usize;
        c::signal(c::SIGINT, handler_ptr);
        c::signal(c::SIGTSTP, handler_ptr);
        c::signal(c::SIGTTIN, c::SIG_IGN);
        c::signal(c::SIGTTOU, c::SIG_IGN);
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
            c::kill(-job.pid, c::SIGHUP);
            c::kill(-job.pid, c::SIGCONT);
        }
    }

    std::process::exit(final_status);
}
