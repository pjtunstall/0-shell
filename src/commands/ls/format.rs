use std::fs::{DirEntry, Metadata};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    collections::VecDeque,
    fs,
    path::{MAIN_SEPARATOR_STR, Path},
};

use chrono;
use terminal_size::{Width, terminal_size};

use super::system;
use crate::ansi::{BLUE, RESET_FG};

fn blue(text: &str) -> String {
    // Use foreground reset (39m) so we don't clear other active attributes (e.g., bold).
    format!("{BLUE}{text}{RESET_FG}")
}

struct FileInfo {
    file_type: String,
    permissions: String,
    hard_links: u64,
    user_name: String,
    group_name: String,
    size: u64,
    timestamp: String,
    name: String,
    suffix: String,
    extended_attr: String,
    symlink: String,
}

impl FileInfo {
    fn format(&self) -> String {
        format!(
            "{}{:>7}{} {:>1} {:>7} {:>6} {:>6} {:>13} {}{} {}",
            self.file_type,
            self.permissions,
            self.extended_attr,
            self.hard_links,
            self.user_name,
            self.group_name,
            self.size,
            self.timestamp,
            self.name.replace(" ", "*"),
            self.suffix,
            self.symlink
        )
    }
}

pub fn get_short_list(flags: u8, path: &Path, is_redirect: bool) -> Result<String, String> {
    let mut entries: VecDeque<String> = match fs::read_dir(path) {
        Ok(dir) => dir
            .filter_map(|entry| match entry {
                Ok(e) => {
                    let name = e.file_name().to_string_lossy().to_string();

                    if flags & 1 == 0 && system::is_hidden(&e.path()) {
                        return None;
                    }

                    let suffix = if flags & 4 != 0 {
                        system::classify(&e.path())
                    } else {
                        String::new()
                    };

                    let display_name = if !is_redirect && e.path().is_dir() {
                        blue(&name)
                    } else {
                        name
                    };

                    Some(format!("{}{}", display_name, suffix))
                }
                Err(e) => Some(format!("Error reading entry: {}", e)),
            })
            .collect(),
        Err(e) => return Ok(format!("Error reading directory: {}", e)),
    };

    if flags & 1 == 1 {
        entries.push_front(String::from(".."));
        entries.push_front(String::from("."));
    }

    if entries.is_empty() {
        return Ok(String::new());
    }

    let mut entries: Vec<_> = entries.into();
    entries.sort();

    short_format_list(entries, is_redirect)
}

pub fn short_format_list(entries: Vec<String>, is_redirect: bool) -> Result<String, String> {
    let mut output = String::new();

    if is_redirect {
        output = entries.join("\n");
        output.push_str("\n");
    } else {
        let term_width = get_terminal_width();
        let max_visible_len = entries.iter().map(|s| visible_width(s)).max().unwrap_or(0);
        let col_width = max_visible_len + 6;

        let num_cols = term_width / col_width;
        let num_rows = (entries.len() + num_cols - 1) / num_cols;

        for row in 0..num_rows {
            for col in 0..num_cols {
                if let Some(entry) = entries.get(row + col * num_rows) {
                    let pad = col_width.saturating_sub(visible_width(entry));
                    output.push_str(entry);
                    output.push_str(&" ".repeat(pad));
                } else {
                    output.push_str(&" ".repeat(col_width));
                }
            }
            output.push('\n');
        }
    }

    Ok(output)
}

fn visible_width(s: &str) -> usize {
    let mut width = 0;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip ANSI escape sequences.
            while let Some(next) = chars.next() {
                if next == 'm' {
                    break;
                }
            }
            continue;
        }
        width += 1;
    }
    width
}

fn get_terminal_width() -> usize {
    if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        80
    }
}

pub fn get_long_list(flags: u8, path: &Path, colorize: bool) -> Result<String, String> {
    let metadata = match fs::metadata(path) {
        Ok(meta) => meta,
        Err(_) => return Ok(String::new()),
    };

    let mut entries: VecDeque<String> = if metadata.is_dir() {
        match fs::read_dir(path) {
            Ok(entries) => entries
                .filter_map(|entry| format_entry_from_direntry(entry.ok()?, flags, colorize))
                .filter(|entry_str| {
                    let name = entry_str.split_whitespace().last().unwrap_or("");
                    flags & 1 == 1 || !system::is_hidden(Path::new(name))
                })
                .collect(),
            Err(_) => return Ok(String::new()),
        }
    } else {
        let file_name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());

        match format_entry_from_path(path, &file_name, flags, colorize) {
            Some(entry_str) => VecDeque::from([entry_str]),
            None => return Ok(String::new()),
        }
    };

    if flags & 1 == 1 {
        if let Ok(absolute_path) = fs::canonicalize(path) {
            let parent_path = absolute_path
                .parent()
                .unwrap_or(Path::new(MAIN_SEPARATOR_STR));
            if let Some(entry) = format_entry_from_path(parent_path, "..", flags, colorize) {
                entries.push_front(entry);
            }
        }

        if let Some(entry) = format_entry_from_path(path, ".", flags, colorize) {
            entries.push_front(entry);
        }
    }

    if entries.is_empty() {
        return Ok(String::new());
    }

    let total = system::get_total_blocks_in_directory(path);
    long_format_list(entries.into_iter().collect(), total)
}

fn long_format_list(entries: Vec<String>, total: u64) -> Result<String, String> {
    if entries.is_empty() {
        return Ok(String::new());
    }

    let mut parsed_entries: Vec<Vec<String>> = entries
        .iter()
        .map(|entry| entry.split_whitespace().map(String::from).collect())
        .collect();

    let column_count = parsed_entries
        .iter()
        .map(|row| row.len())
        .max()
        .unwrap_or(0);
    let mut column_widths = vec![0; column_count];

    for row in &parsed_entries {
        for (i, field) in row.iter().enumerate() {
            if i < column_count {
                column_widths[i] = column_widths[i].max(field.len());
            }
        }
    }

    parsed_entries.sort_by_key(|row| {
        let name = row.last().cloned().unwrap_or_default();
        if name == "./" {
            String::from("\x00")
        } else if name == "../" {
            String::from("\x01")
        } else {
            name
        }
    });

    let formatted_entries: Vec<String> = parsed_entries
        .iter()
        .map(|row| {
            let mut result = String::new();

            // Process each field with the appropriate alignment:
            // - Permissions (col 0): left-aligned
            // - Link count (col 1): right-aligned
            // - Owner/group (cols 2-3): left-aligned
            // - Size (col 4): right-aligned
            // - Date (cols 5-7): specific spacing
            // - Filename (last col): no padding

            for (i, field) in row.iter().enumerate() {
                if i > 0 {
                    result.push(' ');
                }

                if i == 0 || i == 2 {
                    // Left-align permissions, owner, group.
                    result.push_str(&format!("{:<width$}", field, width = column_widths[i]));
                } else if i == 3 {
                    result.push_str(&format!(" {:<width$} ", field, width = column_widths[i]));
                } else if i == 1 || i == 4 {
                    // Right-align link count and size.
                    result.push_str(&format!("{:>width$}", field, width = column_widths[i]));
                } else if i >= 5 && i < row.len() - 1 {
                    // Date fields: standard spacing.
                    result.push_str(&format!("{:<width$}", field, width = column_widths[i]));
                } else if i == row.len() - 1 {
                    // Filename: no padding.
                    result.push_str(field);
                }
            }

            result.replace("*", " ")
        })
        .collect();

    let total = format!("total: {}\n", total.to_string());
    let mut result = total + &formatted_entries.join("\n");
    result.push('\n');

    Ok(result)
}

fn format_entry<T: AsRef<Path>>(
    path: T,
    name: Option<String>,
    metadata: Metadata,
    flags: u8,
    colorize: bool,
) -> Option<String> {
    let path = path.as_ref();
    let name_raw = name.unwrap_or_else(|| {
        path.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    });
    let name = if colorize && metadata.is_dir() {
        blue(&name_raw)
    } else {
        name_raw
    };

    let file_type = if metadata.is_dir() { "d" } else { "-" };
    let (permissions, hard_links, user_name, group_name) =
        system::get_platform_specific_info(&metadata);

    let suffix = if flags & 4 != 0 {
        system::classify(path)
    } else {
        String::new()
    };

    let info = FileInfo {
        file_type: String::from(file_type),
        permissions,
        hard_links,
        user_name,
        group_name,
        size: metadata.len(),
        timestamp: format_time(metadata.modified().unwrap_or(UNIX_EPOCH)),
        name,
        suffix,
        extended_attr: system::get_extended_attributes(path),
        symlink: system::get_symlink_info(&metadata),
    };

    Some(info.format())
}

fn format_entry_from_direntry(e: DirEntry, flags: u8, colorize: bool) -> Option<String> {
    let metadata = e.metadata().ok()?;
    format_entry(
        e.path(),
        Some(e.file_name().to_string_lossy().into_owned()),
        metadata,
        flags,
        colorize,
    )
}

fn format_entry_from_path(path: &Path, name: &str, flags: u8, colorize: bool) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    format_entry(path, Some(String::from(name)), metadata, flags, colorize)
}

fn format_time(modified: SystemTime) -> String {
    let duration = modified.duration_since(UNIX_EPOCH).unwrap();
    let secs = duration.as_secs();
    let timestamp = chrono::DateTime::from_timestamp(secs as i64, 0)
        .map(|naive| naive.format("%b %e %H:%M").to_string())
        .unwrap_or_else(|| format!("{}", secs));

    timestamp
}
