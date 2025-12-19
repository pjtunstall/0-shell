use std::collections::VecDeque;

use crate::{
    c::{self, SIGINT, SIGTSTP},
    commands::{
        self,
        jobs::{self, Job},
    },
    error, input,
};

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
            if c::tcgetattr(c::STDIN_FILENO, &mut tio) == 0 {
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
                c::tcsetattr(c::STDIN_FILENO, c::TCSANOW, &saved);
            }
        }
    }
}

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

pub fn repl() {
    let mut jobs: Vec<Job> = Vec::new();
    let mut current: usize = 0;
    let mut previous: usize = 0;

    unsafe {
        c::signal(SIGINT, c::handle_forwarding);
        c::signal(SIGTSTP, c::handle_forwarding);
    }

    let _style = TextStyle::new();
    let _tty_guard = TtyGuard::new();
    let mut history = VecDeque::new();
    history.push_back(String::new());

    loop {
        jobs::check_background_jobs(&mut jobs, &mut current, &mut previous);

        let input_string = match input::get_input(&mut history) {
            Ok(Some(ok_input)) => ok_input,
            Ok(None) => break,
            Err(err) => {
                let text = format!("0-shell: Failed to get input: {}", err);
                error::red_println(&text);
                continue;
            }
        };

        if input_string.is_empty() {
            continue;
        }

        history.push_back(input_string.clone());

        let input_after_splitting: Vec<String>;
        match input::split::split(&input_string) {
            Ok(res) => {
                input_after_splitting = res;
            }
            Err(err) => {
                error::red_println(&format!("0-shell: {}", &err));
                continue;
            }
        }

        if input_after_splitting.is_empty() {
            error::red_println(&format!("0-shell: Parse error near `\\n'"));
            continue;
        }

        commands::run_command(
            &input_after_splitting,
            &mut jobs,
            &mut current,
            &mut previous,
        );
    }
}
