use std::{env, fs::OpenOptions, io::Write};

use serde_json::de::from_str;

pub fn echo(input: &[String]) -> Result<String, String> {
    debug_assert!(!input.is_empty(), "Input for `echo` should not be empty");
    debug_assert!(
        input[0] == "echo",
        "Input for `{}` should not be passed to `echo`",
        input[0]
    );

    // Because we may want to drain the input
    let input: Vec<String> = input.iter().map(|s| s.to_string()).collect();

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
        } else {
            return Err("Parse error near `\\n'".to_string()); // This should never happen now, thanks to `split`
        }
        input.drain(pos..);
    }

    let mut output = String::new();
    for (i, arg) in input[1..].iter().enumerate() {
        if i > 0 {
            output.push(' ');
        }

        if arg.len() > 1 && (arg.starts_with('"') && arg.ends_with('"'))
            || (arg.starts_with('\'') && arg.ends_with('\''))
        {
            let inside = &arg[1..arg.len() - 1];
            output.push_str(&process_backslashes(inside, 1));
        } else {
            output.push_str(&process_backslashes(arg, 0));
        }
    }

    let json_output = format!("\"{}\"", output);
    output = from_str::<String>(&json_output).map_err(|e| {
        e.to_string()
            .split(" (os ")
            .next()
            .unwrap_or(" ")
            .to_string()
    })?;

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
    *output = output.replace("$USER", &env::var("USER").unwrap_or_default());
    *output = output.replace("$HOSTNAME", &env::var("HOSTNAME").unwrap_or_default());
    *output = output.replace("$PID", &env::var("PID").unwrap_or_default());
    *output = output.replace("$PATH", &env::var("PATH").unwrap_or_default());
    *output = output.replace("$SHELL", &env::var("SHELL").unwrap_or_default());
    *output = output.replace("$UMASK", &env::var("UMASK").unwrap_or_default());
    *output = output.replace("$HOME", &env::var("HOME").unwrap_or_default());
    *output = output.replace("$LANG", &env::var("LANG").unwrap_or_default());
    *output = output.replace("$TERM", &env::var("TERM").unwrap_or_default());
}

fn handle_redirection(output: &str, filename: &str, append: bool) -> Result<String, String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(append)
        .truncate(!append)
        .open(filename)
        .map_err(|e| e.to_string())?;

    file.write_all(output.as_bytes())
        .map_err(|e| e.to_string())?;

    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use std::{env, fs};

    use uuid::Uuid;

    use super::echo;

    #[test]
    fn test_basic_echo() {
        assert_eq!(
            echo(&vec!["echo".to_string(), "hello".to_string()]),
            Ok("hello\n".to_string()),
            "Expected to echo one word"
        );
        assert_eq!(
            echo(&vec![
                "echo".to_string(),
                "hello".to_string(),
                "world".to_string()
            ]),
            Ok("hello world\n".to_string()),
            "Expected to echo two words"
        );
        assert_eq!(
            echo(&vec![
                "echo".to_string(),
                "hello".to_string(),
                "world".to_string(),
                "hello".to_string()
            ]),
            Ok("hello world hello\n".to_string()),
            "Expected to echo three words"
        );
    }

    #[test]
    fn test_special_characters() {
        assert_eq!(
            echo(&vec!["echo".to_string(), "a\\na".to_string()]),
            Ok("ana\n".to_string()),
            "Expected to convert `\\n` to `n`"
        );
        assert_eq!(
            echo(&vec!["echo".to_string(), "a\\\\na".to_string()]),
            Ok("a\na\n".to_string()),
            "Expected to convert `\\\\n` to `\\n`"
        );
        assert_eq!(
            echo(&vec!["echo".to_string(), "a\\\\\\na".to_string()]),
            Ok("a\na\n".to_string()),
            "Expected to convert `\\\\\\n` to `\\n`"
        );
        assert_eq!(
            echo(&vec!["echo".to_string(), "a\\\\\\\\na".to_string()]),
            Ok("a\\na\n".to_string()),
            "Expected to convert `\\\\\\\\n` to `\\\\n`"
        );

        assert_eq!(
            echo(&vec!["echo".to_string(), "\"a\\na\"".to_string()]),
            Ok("a\na\n".to_string()),
            "Expected to leave `\\n` unchanged in quotes"
        );
        assert_eq!(
            echo(&vec!["echo".to_string(), "\"a\\\\na\"".to_string()]),
            Ok("a\na\n".to_string()),
            "Expected to leave `\\\\n` unchanged in quotes"
        );
        assert_eq!(
            echo(&vec!["echo".to_string(), "\"a\\\\\\na\"".to_string()]),
            Ok("a\\na\n".to_string()),
            "Expected to convert `\\\\\\n` in quotes to `\\n`"
        );
        assert_eq!(
            echo(&vec!["echo".to_string(), "\"a\\\\\\\\na\"".to_string()]),
            Ok("a\\na\n".to_string()),
            "Expected to convert `\\\\\\\\n` in quotes to `\\n`"
        );
    }

    #[test]
    fn test_redirection_in_double_quotes() {
        assert_eq!(
            echo(&vec!["echo".to_string(), "\">\"".to_string()]),
            Ok(">\n".to_string()),
            "Expected to escape `>` in double quotes, and give no error when final"
        );

        assert_eq!(
            echo(&vec!["echo".to_string(), "\">>\"".to_string()]),
            Ok(">>\n".to_string()),
            "Expected to leave `>>` unchanged in double quotes, and give no error when final"
        );
    }

    #[test]
    fn test_redirection_in_single_quotes() {
        assert_eq!(
            echo(&vec!["echo".to_string(), "\'>\'".to_string()]),
            Ok(">\n".to_string()),
            "Expected to leave `>` unchanged in single quotes, and give no error when final"
        );

        assert_eq!(
            echo(&vec!["echo".to_string(), "\'>>\'".to_string()]),
            Ok(">>\n".to_string()),
            "Expected to leave `>>` unchanged in single quotes, and give no error when final"
        );
    }

    #[test]
    fn test_env_var_set() {
        let prev_user = env::var("USER").ok();
        unsafe {
            env::set_var("USER", "testuser");
        }

        assert_eq!(
            echo(&vec!["echo".to_string(), "$USER".to_string()]),
            Ok("testuser\n".to_string()),
            "Expected `USER` to be replaced with `testuser`"
        );

        if let Some(value) = prev_user {
            unsafe {
                env::set_var("USER", value);
            }
        } else {
            unsafe {
                env::remove_var("USER");
            }
        }
    }

    #[test]
    fn test_env_var_unset() {
        let prev_lang = env::var("LANG").ok();
        unsafe {
            env::remove_var("LANG");
        }
        assert_eq!(
            echo(&vec!["echo".to_string(), "$LANG".to_string()]),
            Ok("\n".to_string()),
            "Expected empty substitution when LANG is unset"
        );
        if let Some(value) = prev_lang {
            unsafe {
                env::set_var("LANG", value);
            }
        } else {
            unsafe {
                env::remove_var("LANG");
            }
        }
    }

    #[test]
    fn test_write_to_file() {
        let file = Uuid::new_v4().to_string();
        let mut expected = "hello\n";
        let mut output = echo(&vec![
            "echo".to_string(),
            "hello".to_string(),
            ">".to_string(),
            file.clone(),
        ]);
        assert!(output.unwrap().is_empty());
        let mut contents = fs::read_to_string(&file).expect("Failed to read file");
        assert_eq!(contents, expected, "Expected to write to nonexistent file");

        expected = "world\n";
        output = echo(&vec![
            "echo".to_string(),
            "world".to_string(),
            ">".to_string(),
            file.clone(),
        ]);
        assert!(output.unwrap().is_empty());
        contents = fs::read_to_string(&file).expect("Failed to read file");
        assert_eq!(contents, expected, "Expected to overwrite existing file");

        fs::remove_file(file).ok();
    }

    #[test]
    fn test_append_to_file() {
        let file = Uuid::new_v4().to_string();
        let mut expected = "hello\n";
        let mut output = echo(&vec![
            "echo".to_string(),
            "hello".to_string(),
            ">>".to_string(),
            file.clone(),
        ]);
        assert!(output.is_ok());
        let mut contents = fs::read_to_string(&file).expect("Failed to read file");
        assert_eq!(contents, expected, "Expected to append to nonexistent file");

        expected = "hello\nworld\n";
        output = echo(&vec![
            "echo".to_string(),
            "world".to_string(),
            ">>".to_string(),
            file.clone(),
        ]);
        assert!(output.unwrap().is_empty());
        contents = fs::read_to_string(&file).expect("Failed to read file");
        assert_eq!(contents, expected, "Expected to append to existing file");

        fs::remove_file(file).ok();
    }

    #[test]
    fn test_ignore_write_to_multiple_files() {
        let file1 = Uuid::new_v4().to_string();
        let file2 = Uuid::new_v4().to_string();
        let expected = "hello\n";
        let output = echo(&vec![
            "echo".to_string(),
            "hello".to_string(),
            ">".to_string(),
            file1.clone(),
            file2.clone(),
        ]);
        assert!(output.unwrap().is_empty());
        let contents = fs::read_to_string(&file1).expect("Failed to read file");
        assert_eq!(
            contents, expected,
            "Expected to write to only one file when two names are given"
        );

        fs::remove_file(file1).ok();
        fs::remove_file(file2).ok();
    }

    #[test]
    fn test_ignore_append_to_multiple_files() {
        let file1 = Uuid::new_v4().to_string();
        let file2 = Uuid::new_v4().to_string();
        let expected = "hello\n";
        let output = echo(&vec![
            "echo".to_string(),
            "hello".to_string(),
            ">>".to_string(),
            file1.clone(),
            file2.clone(),
        ]);
        assert!(output.unwrap().is_empty());
        let contents = fs::read_to_string(&file1).expect("Failed to read file");
        assert_eq!(
            contents, expected,
            "Expected to write to only one file when two names are given"
        );

        fs::remove_file(file1).ok();
        fs::remove_file(file2).ok();
    }
}
