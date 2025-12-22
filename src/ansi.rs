pub const ANSI_PREFIX: char = '\x1b';

pub const ERROR_COLOR: &str = RED;
pub const USER_COLOR: &str = PURPLE;
pub const FOLDER_COLOR: &str = BRIGHT_CYAN;

const RED: &str = "\x1b[31m";
const PURPLE: &str = "\x1b[35m";
const BRIGHT_CYAN: &str = "\x1b[96m";

pub const RESET: &str = "\x1b[0m";
pub const RESET_FG: &str = "\x1b[39m";

pub const BOLD: &str = "\x1b[1m";

pub const CLEAR_LINE: &str = "\x1b[K";

pub fn cursor_up(rows: usize) -> String {
    format!("\x1b[{}A", rows)
}
