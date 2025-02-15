use std::env;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};

use serde_json::de::from_str;

pub fn echo(input: &Vec<String>) -> Result<String, String> {
    if input.len() < 2 {
        return Ok("\n".to_string());
    }

    let mut input = input.clone();

    let mut append = false;
    let mut filename = String::new();
    if let Some(pos) = input.iter().position(|arg| arg == ">" || arg == ">>") {
        if input[pos] == ">>" {
            append = true;
        }
        if pos + 1 < input.len() {
            filename = input[pos + 1].clone();
        }
        input.drain(pos..);
    }

    let mut output = String::new();
    for (i, arg) in input[1..].iter().enumerate() {
        if i > 0 {
            output.push(' ');
        }

        if arg.starts_with('"') && arg.ends_with('"') {
            let inside = &arg[1..arg.len() - 1];
            output.push_str(&process_backslashes(inside, 1));
        } else {
            output.push_str(&process_backslashes(arg, 0));
        }
    }

    let json_output = format!("\"{}\"", output);
    output = from_str::<String>(&json_output).map_err(|e| e.to_string())?;

    parse_environment_variables(&mut output);
    output.push('\n');

    if filename.is_empty() {
        Ok(output)
    } else {
        handle_redirection(&output, &filename, append)
    }
}

fn process_backslashes(s: &str, plus: usize) -> String {
    let mut result = String::new();
    let mut backslash_count = 0;

    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            backslash_count += 1;
        } else {
            if backslash_count > 0 {
                let keep_backslashes = (backslash_count + plus) / 2;
                result.push_str(&"\\".repeat(keep_backslashes));
                backslash_count = 0;
            }
            result.push(c);
        }
    }

    // In case the string ends with backslashes
    if backslash_count > 0 {
        let keep_backslashes = (backslash_count + 1) / 2;
        result.push_str(&"\\".repeat(keep_backslashes));
    }

    result
}

fn parse_environment_variables(output: &mut String) {
    *output = output.replace(
        "$USER",
        &env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
    );
    *output = output.replace(
        "$HOSTNAME",
        &env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string()),
    );
    *output = output.replace(
        "$PID",
        &env::var("PID").unwrap_or_else(|_| "unknown".to_string()),
    );
    *output = output.replace(
        "$PATH",
        &env::var("PATH").unwrap_or_else(|_| "unknown".to_string()),
    );
    *output = output.replace(
        "$SHELL",
        &env::var("SHELL").unwrap_or_else(|_| "unknown".to_string()),
    );
    *output = output.replace(
        "$UMASK",
        &env::var("UMASK").unwrap_or_else(|_| "unknown".to_string()),
    );
    *output = output.replace(
        "$HOME",
        &env::var("HOME").unwrap_or_else(|_| "unknown".to_string()),
    );
    *output = output.replace(
        "$LANG",
        &env::var("LANG").unwrap_or_else(|_| "unknown".to_string()),
    );
    *output = output.replace(
        "$TERM",
        &env::var("TERM").unwrap_or_else(|_| "unknown".to_string()),
    );
}

fn handle_redirection(output: &str, filename: &str, append: bool) -> Result<String, String> {
    let file = if append {
        OpenOptions::new()
            .append(true)
            .create(true)
            .open(&filename)
            .map_err(|e| e.to_string())?
    } else {
        File::create(&filename).map_err(|e| e.to_string())?
    };

    let mut writer = BufWriter::new(file);
    writer
        .write_all(output.as_bytes())
        .map_err(|e| e.to_string())?;

    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_echo() {
        assert_eq!(
            echo(&vec!["echo".to_string(), "hello".to_string()]),
            Ok("hello\n".to_string())
        );
        assert_eq!(
            echo(&vec![
                "echo".to_string(),
                "hello".to_string(),
                "world".to_string()
            ]),
            Ok("hello world\n".to_string())
        );
        assert_eq!(
            echo(&vec![
                "echo".to_string(),
                "hello".to_string(),
                "world".to_string(),
                "hello".to_string()
            ]),
            Ok("hello world hello\n".to_string())
        );
    }
}
