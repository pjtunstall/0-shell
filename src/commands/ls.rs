#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{collections::VecDeque, fs, path::Path};

use terminal_size::{terminal_size, Width};

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

    let result;

    if first_pathname_index == 0 {
        result = process_path(flags, Path::new("."));
    } else {
        result = process_path(flags, Path::new(&input[first_pathname_index]));
    }

    result
}

fn process_path(flags: u8, path: &Path) -> Result<String, String> {
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
                let suffix = classify(&e.path());
                format!("{}{}", name, suffix)
            })
        })
        .filter(|name| flags & 1 == 1 || !name.starts_with('.'))
        .collect();

    if flags & 1 == 1 {
        entries.push_front(".".to_string());
        entries.push_front("..".to_string());
    }

    if entries.is_empty() {
        return Ok(String::new());
    }

    let mut entries: Vec<_> = entries.into();
    entries.sort();

    format_list(entries)
}

fn format_list(entries: Vec<String>) -> Result<String, String> {
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
        "/".to_string()
    } else if path.is_symlink() {
        "@".to_string()
    } else {
        #[cfg(unix)]
        {
            if path
                .metadata()
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
            {
                "*".to_string()
            } else {
                "".to_string()
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
            "".to_string()
        }
    }
}

fn get_terminal_width() -> usize {
    if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        80
    }
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
