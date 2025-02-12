use std::fs;
use terminal_size::{terminal_size, Width};

pub fn ls(_input: &Vec<String>) -> Result<String, String> {
    let mut entries: Vec<String> = fs::read_dir(".")
        .map_err(|_| "ls: cannot open directory '.': permission denied".to_string())?
        .filter_map(|entry| {
            entry
                .ok()
                .map(|e| e.file_name().to_string_lossy().to_string())
        })
        .filter(|name| !name.starts_with('.'))
        .collect();

    if entries.is_empty() {
        return Ok(String::new());
    }

    entries.sort();

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
        if row < num_rows - 1 {
            output.push('\n');
        }
    }

    Ok(output)
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
