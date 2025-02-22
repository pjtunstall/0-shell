use std::fs::{DirEntry, Metadata};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    collections::VecDeque,
    fs,
    path::{Path, MAIN_SEPARATOR_STR},
};

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

use chrono;
use terminal_size::{terminal_size, Width};
use users::{get_group_by_gid, get_user_by_uid};
use xattr;

const USAGE: &str = "Usage: ls [-a] [-l] [-F] [DIRECTORY]...";

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

    if first_pathname_index == 0 {
        let path = Path::new(".");
        if flags & 2 != 0 {
            return get_long_list(flags, path);
        } else {
            return get_short_list(flags, path);
        }
    }

    let mut files = Vec::new();
    let mut dirs = Vec::new();
    let mut non_paths = Vec::new();

    // Separate directories from regular files
    for arg in &input[first_pathname_index..] {
        let path = Path::new(arg);
        if path.is_dir() {
            dirs.push(arg);
        } else {
            match path.exists() {
                true => files.push(arg.to_string()),
                false => non_paths.push(
                    format!(
                        "\x1b[31m{}: No such file or directory found\x1b[0m\x1b[1m\n",
                        arg
                    )
                    .to_string(),
                ),
            }
        }
    }

    non_paths.sort();
    files.sort();

    let mut results = String::new();

    for item in &non_paths {
        results.push_str(&item);
    }

    if !files.is_empty() {
        if flags & 2 != 0 {
            for file in &files {
                let file_path = Path::new(file);
                let file_listing = get_long_list(flags, file_path)?;
                results.push_str(&file_listing);
            }
        } else {
            let formatted_files = short_format_list(files.clone())?;
            results.push_str(&formatted_files);
        }
    }

    // Process directories
    for (i, dir) in dirs.iter().enumerate() {
        let path = Path::new(dir);

        // Add spacing between sections
        if i > 0 || !files.is_empty() {
            results.push_str("\n");
        }

        // Print directory header if multiple directories or if we had non-dir files
        if input.len() > 2 {
            results.push_str(&format!("{}:\n", dir));
        }

        let dir_listing = if flags & 2 != 0 {
            get_long_list(flags, path)?
        } else {
            get_short_list(flags, path)?
        };

        results.push_str(&dir_listing);
    }

    Ok(results)
}

fn get_platform_specific_info(metadata: &Metadata) -> (String, u64, String, String) {
    #[cfg(unix)]
    {
        let permissions = mode_to_string(metadata.mode());
        let hard_links = metadata.nlink();
        let (owner, group) = (metadata.uid(), metadata.gid());
        let (user_name, group_name) = get_user_and_group(owner, group);
        (permissions, hard_links, user_name, group_name)
    }

    #[cfg(windows)]
    {
        (
            "rw-r--r--".to_string(),
            1,
            "Owner".to_string(),
            "Group".to_string(),
        )
    }
}

fn get_extended_attributes(path: &Path) -> String {
    #[cfg(unix)]
    {
        if has_extended_attributes(path) {
            "@".to_string()
        } else {
            String::new()
        }
    }

    #[cfg(windows)]
    {
        String::new()
    }
}

fn get_symlink_info(metadata: &Metadata) -> String {
    #[cfg(unix)]
    {
        if metadata.file_type().is_symlink() {
            "-> symlink".to_string()
        } else {
            String::new()
        }
    }

    #[cfg(windows)]
    {
        String::new()
    }
}

fn format_entry<T: AsRef<Path>>(
    path: T,
    name: Option<String>,
    metadata: Metadata,
    flags: u8,
) -> Option<String> {
    let path = path.as_ref();
    let name = name.unwrap_or_else(|| {
        path.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    });

    let file_type = if metadata.is_dir() { "d" } else { "-" };
    let (permissions, hard_links, user_name, group_name) = get_platform_specific_info(&metadata);

    let suffix = if flags & 4 != 0 {
        classify(path)
    } else {
        String::new()
    };

    let info = FileInfo {
        file_type: file_type.to_string(),
        permissions,
        hard_links,
        user_name,
        group_name,
        size: metadata.len(),
        timestamp: format_time(metadata.modified().unwrap_or(UNIX_EPOCH)),
        name,
        suffix,
        extended_attr: get_extended_attributes(path),
        symlink: get_symlink_info(&metadata),
    };

    Some(info.format())
}

fn format_entry_from_direntry(e: DirEntry, flags: u8) -> Option<String> {
    let metadata = e.metadata().ok()?;
    format_entry(
        e.path(),
        Some(e.file_name().to_string_lossy().into_owned()),
        metadata,
        flags,
    )
}

fn format_entry_from_path(path: &Path, name: &str, flags: u8) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    format_entry(path, Some(name.to_string()), metadata, flags)
}

fn get_short_list(flags: u8, path: &Path) -> Result<String, String> {
    let mut entries: VecDeque<String> = match fs::read_dir(path) {
        Ok(dir) => dir
            .filter_map(|entry| match entry {
                Ok(e) => {
                    let name = e.file_name().to_string_lossy().to_string();

                    // Ensure hidden files are correctly filtered on Windows
                    if flags & 1 == 0 && is_hidden(&e.path()) {
                        return None;
                    }

                    let suffix = if flags & 4 != 0 {
                        classify(&e.path())
                    } else {
                        String::new()
                    };

                    Some(format!("{}{}", name, suffix))
                }
                Err(e) => Some(format!("Error reading entry: {}", e)),
            })
            .collect(),
        Err(e) => return Ok(format!("Error reading directory: {}", e)),
    };

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

fn is_hidden(path: &Path) -> bool {
    #[cfg(unix)]
    {
        path.file_name()
            .map(|name| name.to_string_lossy().starts_with('.'))
            .unwrap_or(false)
    }

    #[cfg(target_os = "windows")]
    {
        use winapi::um::winnt::FILE_ATTRIBUTE_HIDDEN;

        fs::metadata(path)
            .map(|metadata| metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0)
            .unwrap_or(false)
    }
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
    let metadata = match fs::metadata(path) {
        Ok(meta) => meta,
        Err(_) => return Ok(String::new()),
    };

    let mut entries: VecDeque<String> = if metadata.is_dir() {
        match fs::read_dir(path) {
            Ok(entries) => entries
                .filter_map(|entry| format_entry_from_direntry(entry.ok()?, flags))
                .filter(|entry_str| {
                    let name = entry_str.split_whitespace().last().unwrap_or("");
                    flags & 1 == 1 || !is_hidden(Path::new(name))
                })
                .collect(),
            Err(_) => return Ok(String::new()),
        }
    } else {
        let file_name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());

        match format_entry_from_path(path, &file_name, flags) {
            Some(entry_str) => VecDeque::from([entry_str]),
            None => return Ok(String::new()),
        }
    };

    if flags & 1 == 1 {
        if let Ok(absolute_path) = fs::canonicalize(path) {
            let parent_path = absolute_path
                .parent()
                .unwrap_or(Path::new(MAIN_SEPARATOR_STR));
            if let Some(entry) = format_entry_from_path(parent_path, "..", flags) {
                entries.push_front(entry);
            }
        }

        if let Some(entry) = format_entry_from_path(path, ".", flags) {
            entries.push_front(entry);
        }
    }

    if entries.is_empty() {
        return Ok(String::new());
    }

    let total = get_total_blocks_in_directory(path);
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

    let total = format!("total: {}\n", total.to_string());
    let mut result = total + &formatted_entries.join("\n");
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

fn get_total_blocks_in_directory(path: &Path) -> u64 {
    let mut total_blocks = 0;

    if path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.filter_map(Result::ok) {
                let metadata = entry.metadata().unwrap();
                let blocks = metadata.blocks(); // Returns the number of 512-byte blocks
                total_blocks += blocks;
            }
        }
    }

    total_blocks
}

#[cfg(test)]
mod tests {
    use super::ls;

    #[test]
    fn test_ls() {
        let basic = vec!["ls".to_string()];
        let a = vec!["ls".to_string(), "-a".to_string()];
        let l = vec!["ls".to_string(), "-l".to_string()];
        let f = vec!["ls".to_string(), "-F".to_string()];
        let alf = vec!["ls".to_string(), "-alF".to_string()];
        let l_a_f = vec![
            "ls".to_string(),
            "-l".to_string(),
            "-a".to_string(),
            "-F".to_string(),
        ];

        let inputs = [basic, a, l, f, alf, l_a_f];

        for input in inputs {
            assert!(ls(&input).is_ok(), "`ls` should be ok for {:?}", input);
        }
    }
}
