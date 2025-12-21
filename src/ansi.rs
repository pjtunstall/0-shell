pub const ANSI_PREFIX: char = '\x1b';

pub const RED: &str = "\x1b[31m";
pub const BLUE: &str = "\x1b[34m";
pub const BRIGHT_GREEN: &str = "\x1b[92m";

pub const RESET: &str = "\x1b[0m";
pub const RESET_FG: &str = "\x1b[39m";

pub const BOLD: &str = "\x1b[1m";

pub const CLEAR_LINE: &str = "\x1b[K";

pub fn cursor_up(rows: usize) -> String {
    format!("\x1b[{}A", rows)
}
