use std::fs::DirEntry;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::VecDeque, fs, path::Path};

use chrono;
use terminal_size::{terminal_size, Width};
use users::{get_group_by_gid, get_user_by_uid};
use xattr;

const USAGE: &str = "Usage: ls [-a] [-l] [-F] [DIRECTORY]...";

pub fn ls(input: &Vec<String>) -> Result<String, String> {
    debug_assert!(!input.is_empty(), "Input for `ls` should not be empty");
    debug_assert!(
        input[0] == "ls",
        "Input for `{}` should not be passed to `ls`",
        input[0]
    );

    let mut flags: u8 = 0;
    let mut first_pathname_index: usize = 0;

    for (i, arg) in input[1..].iter().enumerate() {
        if arg.starts_with('-') {
            if arg.contains('a') {
                flags |= 1;
            }
            if arg.contains('l') {
                flags |= 2;
            }
            if arg.contains('F') {
                flags |= 4;
            }
            if arg.chars().skip(1).any(|c| !['a', 'l', 'F'].contains(&c)) {
                return Err(format!("unrecognized option `{}'\n{}", arg, USAGE).to_string());
            }
        } else {
            first_pathname_index = i + 1;
            debug_assert!(first_pathname_index < input.len());
            break;
        }
    }

    let path = if first_pathname_index == 0 {
        Path::new(".")
    } else {
        Path::new(&input[first_pathname_index])
    };
    check_dir(path)?;

    if flags & 2 != 0 {
        get_long_list(flags, path)
    } else {
        get_short_list(flags, path)
    }
}

fn check_dir(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Err(format!(
            "cannot access `{}': no such file or directory",
            path.display()
        )
        .to_string());
    }
    if !path.is_dir() {
        return Err(format!("`{}' is not a directory", path.display()).to_string());
    }

    Ok(String::new())
}

fn get_short_list(flags: u8, path: &Path) -> Result<String, String> {
    let mut entries: VecDeque<String> = fs::read_dir(path)
        .map_err(|_| {
            format!(
                "ls: cannot open directory `{}': permission denied",
                path.display()
            )
        })?
        .filter_map(|entry| {
            entry.ok().map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                let suffix = if flags & 4 != 0 {
                    classify(&e.path())
                } else {
                    String::new()
                };
                format!("{}{}", name, suffix)
            })
        })
        .filter(|name| flags & 1 == 1 || !name.starts_with('.'))
        .collect();

    if flags & 1 == 1 {
        entries.push_front("..".to_string());
        entries.push_front(".".to_string());
    }

    if entries.is_empty() {
        return Ok(String::new());
    }
    let mut entries: Vec<_> = entries.into();
    entries.sort();

    short_format_list(entries)
}

fn short_format_list(entries: Vec<String>) -> Result<String, String> {
    let term_width = get_terminal_width();
    let max_len = entries.iter().map(|s| s.len()).max().unwrap_or(0);
    let col_width = max_len + 6;

    let num_cols = term_width / col_width;
    let num_rows = (entries.len() + num_cols - 1) / num_cols;

    let mut output = String::new();
    for row in 0..num_rows {
        for col in 0..num_cols {
            if let Some(entry) = entries.get(row + col * num_rows) {
                output.push_str(&format!("{:<width$}", entry, width = col_width));
            } else {
                output.push_str(&" ".repeat(col_width));
            }
        }
        output.push('\n');
    }

    Ok(output)
}

fn classify(path: &Path) -> String {
    if path.is_dir() {
        return "/".to_string();
    } else if path.is_symlink() {
        return "@".to_string();
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;

        if let Ok(metadata) = path.metadata() {
            let file_type = metadata.file_type();

            if metadata.permissions().mode() & 0o111 != 0 {
                return "*".to_string();
            } else if file_type.is_fifo() {
                return "|".to_string();
            } else if file_type.is_socket() {
                return "=".to_string();
            }
        }
    }

    #[cfg(windows)]
    {
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            if ["exe", "bat", "cmd", "com"].contains(&ext.as_str()) {
                return "*".to_string();
            }
        }
    }

    "".to_string()
}

fn get_terminal_width() -> usize {
    if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        80
    }
}

fn get_long_list(flags: u8, path: &Path) -> Result<String, String> {
    let mut entries: VecDeque<String> = fs::read_dir(path)
        .map_err(|_| {
            format!(
                "ls: cannot open directory `{}': permission denied",
                path.display()
            )
        })?
        .filter_map(|entry| format_entry_from_direntry(entry.ok()?, flags))
        .filter(|entry_str| {
            let name = entry_str.split_whitespace().last().unwrap_or(&"");
            flags & 1 == 1 || !name.starts_with('.')
        })
        .collect();

    if flags & 1 == 1 {
        if let Ok(absolute_path) = fs::canonicalize(path) {
            let parent_path = absolute_path.parent().unwrap_or(Path::new("/"));
            let parent_dir_entry = format_entry_from_path(parent_path, "..", flags);
            if let Some(entry) = parent_dir_entry {
                entries.push_front(entry);
            }
        }

        let current_dir_entry = format_entry_from_path(path, ".", flags);
        if let Some(entry) = current_dir_entry {
            entries.push_front(entry);
        }
    }

    if entries.is_empty() {
        return Ok(String::new());
    }

    let entries: Vec<_> = entries.into_iter().collect();

    long_format_list(entries)
}

fn format_entry_from_direntry(e: DirEntry, flags: u8) -> Option<String> {
    let metadata = e.metadata().ok()?;
    let file_type = if metadata.is_dir() { "d" } else { "-" };
    let permissions = mode_to_string(metadata.mode());
    let hard_links = get_hard_links(&e.path()).unwrap_or(0);
    let size = metadata.len();
    let owner = metadata.uid();
    let group = metadata.gid();
    let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
    let timestamp = format_time(modified);
    let (user_name, group_name) = get_user_and_group(owner, group);

    let name = e.file_name().to_string_lossy().to_string();
    let suffix = if flags & 4 != 0 {
        classify(&e.path())
    } else {
        String::new()
    };

    let extended_attr = if has_extended_attributes(&e.path()) {
        "@"
    } else {
        ""
    };

    Some(format!(
        "{}{:>7}{} {:>1} {:>7} {:>6} {:>6} {:>13} {}{}",
        file_type,
        permissions,
        extended_attr,
        hard_links,
        user_name,
        group_name,
        size,
        timestamp,
        name.replace(" ", "*"),
        suffix
    ))
}

fn format_entry_from_path(path: &Path, name: &str, flags: u8) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    let file_type = if metadata.is_dir() { "d" } else { "-" };
    let permissions = mode_to_string(metadata.mode());
    let hard_links = get_hard_links(path).unwrap_or(0);
    let size = metadata.len();
    let owner = metadata.uid();
    let group = metadata.gid();
    let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
    let timestamp = format_time(modified);
    let (user_name, group_name) = get_user_and_group(owner, group);

    let suffix = if flags & 4 != 0 && metadata.is_dir() {
        "/"
    } else {
        ""
    };

    let extended_attr = if has_extended_attributes(path) {
        "@"
    } else {
        ""
    };

    Some(format!(
        "{}{:>7}{} {:>1} {:>7} {:>6} {:>6} {:>13} {}{}",
        file_type,
        permissions,
        extended_attr,
        hard_links,
        user_name,
        group_name,
        size,
        timestamp,
        name,
        suffix
    ))
}

fn long_format_list(entries: Vec<String>) -> Result<String, String> {
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
            "\x00".to_string()
        } else if name == "../" {
            "\x01".to_string()
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
                    // Left-align permissions, owner, group
                    result.push_str(&format!("{:<width$}", field, width = column_widths[i]));
                } else if i == 3 {
                    result.push_str(&format!(" {:<width$} ", field, width = column_widths[i]));
                } else if i == 1 || i == 4 {
                    // Right-align link count and size
                    result.push_str(&format!("{:>width$}", field, width = column_widths[i]));
                } else if i >= 5 && i < row.len() - 1 {
                    // Date fields: standard spacing
                    result.push_str(&format!("{:<width$}", field, width = column_widths[i]));
                } else if i == row.len() - 1 {
                    // Filename: no padding
                    result.push_str(field);
                }
            }

            result.replace("*", " ")
        })
        .collect();

    let mut result = formatted_entries.join("\n");
    result.push('\n');

    Ok(result)
}

fn mode_to_string(mode: u32) -> String {
    let user = (mode >> 6) & 0b111;
    let group = (mode >> 3) & 0b111;
    let other = mode & 0b111;

    let mut perms_str = String::new();

    for &bits in &[user, group, other] {
        perms_str.push(if bits & 0b100 != 0 { 'r' } else { '-' });
        perms_str.push(if bits & 0b010 != 0 { 'w' } else { '-' });
        perms_str.push(if bits & 0b001 != 0 { 'x' } else { '-' });
    }

    perms_str
}

fn get_user_and_group(uid: u32, gid: u32) -> (String, String) {
    let user = get_user_by_uid(uid)
        .map(|u| u.name().to_string_lossy().into_owned())
        .unwrap_or_else(|| format!("{}", uid));
    let group = get_group_by_gid(gid)
        .map(|g| g.name().to_string_lossy().into_owned())
        .unwrap_or_else(|| format!("{}", gid));

    (user, group)
}

fn format_time(modified: SystemTime) -> String {
    let duration = modified.duration_since(UNIX_EPOCH).unwrap();
    let secs = duration.as_secs();
    let timestamp = chrono::DateTime::from_timestamp(secs as i64, 0)
        .map(|naive| naive.format("%b %e %H:%M").to_string())
        .unwrap_or_else(|| format!("{}", secs));

    timestamp
}

fn has_extended_attributes(path: &Path) -> bool {
    match xattr::list(path) {
        Ok(attrs) => attrs.count() > 0,
        Err(_) => false,
    }
}

fn get_hard_links(path: &Path) -> Result<u64, String> {
    let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
    Ok(metadata.nlink())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ls() {
        let result = ls(&vec!["ls".to_string()]);
        assert!(result.is_ok());
    }
}
