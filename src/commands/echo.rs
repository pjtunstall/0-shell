use std::{env, fs::OpenOptions, io::Write};

use serde_json::de::from_str;

use super::redirect;

pub fn echo(input: &[String]) -> Result<String, String> {
    debug_assert!(!input.is_empty(), "Input for `echo` should not be empty");
    debug_assert!(
        input[0] == "echo",
        "Input for `{}` should not be passed to `echo`",
        input[0]
    );

    let (sources, targets) = redirect::separate_sources_from_targets(input);

    if input.len() < 2 {
        return Ok("\n".to_string());
    }

    let mut output = String::new();
    for (i, &arg) in sources.iter().enumerate() {
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

    if targets.is_empty() {
        Ok(output)
    } else {
        handle_redirection(&output, targets)
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

fn handle_redirection(output: &str, targets: Vec<[&String; 2]>) -> Result<String, String> {
    for target in targets {
        let append = target[0] == ">>";

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(append)
            .truncate(!append)
            .open(target[1])
            .map_err(|e| e.to_string())?;

        file.write_all(output.as_bytes())
            .map_err(|e| e.to_string())?;
    }

    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::{env, fs};

    use uuid::Uuid;

    use super::echo;
    use crate::{string_vec, test_helpers::TempStore};

    #[test]
    fn test_basic_echo() {
        assert_eq!(
            echo(&string_vec!["echo", "hello"]),
            Ok("hello\n".to_string()),
            "Expected to echo one word"
        );
        assert_eq!(
            echo(&string_vec!["echo", "hello", "world"]),
            Ok("hello world\n".to_string()),
            "Expected to echo two words"
        );
        assert_eq!(
            echo(&string_vec!["echo", "hello", "world", "hello"]),
            Ok("hello world hello\n".to_string()),
            "Expected to echo three words"
        );
    }

    #[test]
    fn test_special_characters() {
        assert_eq!(
            echo(&string_vec!["echo", "a\\na"]),
            Ok("ana\n".to_string()),
            "Expected to convert `\\n` to `n`"
        );
        assert_eq!(
            echo(&string_vec!["echo", "a\\\\na"]),
            Ok("a\na\n".to_string()),
            "Expected to convert `\\\\n` to `\\n`"
        );
        assert_eq!(
            echo(&string_vec!["echo", "a\\\\\\na"]),
            Ok("a\na\n".to_string()),
            "Expected to convert `\\\\\\n` to `\\n`"
        );
        assert_eq!(
            echo(&string_vec!["echo", "a\\\\\\\\na"]),
            Ok("a\\na\n".to_string()),
            "Expected to convert `\\\\\\\\n` to `\\\\n`"
        );

        assert_eq!(
            echo(&string_vec!["echo", "\"a\\na\""]),
            Ok("a\na\n".to_string()),
            "Expected to leave `\\n` unchanged in quotes"
        );
        assert_eq!(
            echo(&string_vec!["echo".to_string(), "\"a\\\\na\"".to_string()]),
            Ok("a\na\n".to_string()),
            "Expected to leave `\\\\n` unchanged in quotes"
        );
        assert_eq!(
            echo(&string_vec!["echo", "\"a\\\\\\na\""]),
            Ok("a\\na\n".to_string()),
            "Expected to convert `\\\\\\n` in quotes to `\\n`"
        );
        assert_eq!(
            echo(&string_vec!["echo", "\"a\\\\\\\\na\""]),
            Ok("a\\na\n".to_string()),
            "Expected to convert `\\\\\\\\n` in quotes to `\\n`"
        );
    }

    #[test]
    fn test_redirection_in_double_quotes() {
        assert_eq!(
            echo(&string_vec!["echo", "\">\""]),
            Ok(">\n".to_string()),
            "Expected to escape `>` in double quotes, and give no error when final"
        );

        assert_eq!(
            echo(&string_vec!["echo", "\">>\""]),
            Ok(">>\n".to_string()),
            "Expected to leave `>>` unchanged in double quotes, and give no error when final"
        );
    }

    #[test]
    fn test_redirection_in_single_quotes() {
        assert_eq!(
            echo(&string_vec!["echo", "\'>\'"]),
            Ok(">\n".to_string()),
            "Expected to leave `>` unchanged in single quotes, and give no error when final"
        );

        assert_eq!(
            echo(&string_vec!["echo", "\'>>\'"]),
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
            echo(&string_vec!["echo", "$USER"]),
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
            echo(&string_vec!["echo", "$LANG"]),
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
        let mut input = string_vec!["echo", "hello", ">", &file];
        let mut output = echo(&input);
        assert!(output.unwrap().is_empty());
        let mut contents = fs::read_to_string(&file).expect("Failed to read file");
        assert_eq!(contents, expected, "Expected to write to nonexistent file");

        expected = "world\n";
        input = string_vec!["echo", "world", ">", &file];
        output = echo(&input);
        assert!(output.unwrap().is_empty());
        contents = fs::read_to_string(&file).expect("Failed to read file");
        assert_eq!(contents, expected, "Expected to overwrite existing file");

        fs::remove_file(file).ok();
    }

    #[test]
    fn test_append_to_file() {
        let file = Uuid::new_v4().to_string();

        let mut input = string_vec!["echo", "hello", ">>", &file];
        let mut expected = "hello\n";
        let mut output = echo(&input);
        assert!(output.is_ok());
        let mut contents = fs::read_to_string(&file).expect("Failed to read file");
        assert_eq!(contents, expected, "Expected to append to nonexistent file");

        input = string_vec!["echo", "world", ">>", &file];
        expected = "hello\nworld\n";
        output = echo(&input);
        assert!(output.unwrap().is_empty());
        contents = fs::read_to_string(&file).expect("Failed to read file");
        assert_eq!(contents, expected, "Expected to append to existing file");

        fs::remove_file(file).ok();
    }

    #[test]
    fn test_ignore_write_to_multiple_files() {
        let file1 = Uuid::new_v4().to_string();

        let input = string_vec!["echo", "hello", ">", &file1, "file2"];
        let expected = "hello file2\n";
        let output = echo(&input);

        assert!(output.unwrap().is_empty());
        let contents = fs::read_to_string(&file1).expect("Failed to read file");
        assert!(
            !Path::new("file2").exists(),
            "Expected to write to only one file when two names appear after a single write operator"
        );
        assert_eq!(contents, expected, "Contents should include `file2`");

        fs::remove_file(file1).ok();
    }

    #[test]
    fn test_ignore_append_to_multiple_files() {
        let file1 = Uuid::new_v4().to_string();

        let input = string_vec!["echo", "hello", ">>", &file1, "file2"];
        let expected = "hello file2\n";
        let output = echo(&input);
        assert!(output.unwrap().is_empty());
        let contents = fs::read_to_string(&file1).expect("Failed to read file");
        assert!(
            !Path::new("file2").exists(),
            "Expected to write to only one file when two names appear after a single append operator"
        );
        assert_eq!(contents, expected, "Contents should include `file2`");

        fs::remove_file(file1).ok();
    }

    #[test]
    fn test_multiple_redirect_targets() {
        let temp_store = TempStore::new(2);
        let u_str = &temp_store.store[0];
        let v_str = &temp_store.store[1];

        let input: Vec<String> = string_vec!["echo", "hello", ">", u_str, ">", v_str];

        let result = echo(&input);
        assert!(result.is_ok(), "Result of multiple redirects should be ok");

        let u = Path::new(u_str);
        let v = Path::new(v_str);

        assert!(
            u.exists(),
            "1st redirect target file should have been created by `echo`"
        );
        assert!(
            v.exists(),
            "2nd redirect target file should have been created by `echo`"
        );

        let contents_of_u = fs::read_to_string(u).expect("Failed to read 1st redirect target file");
        let contents_of_v = fs::read_to_string(v).expect("Failed to read 2nd redierct target file");

        let expected = "hello\n";

        assert_eq!(
            contents_of_u, expected,
            "Contents of 1st redirect target file should match input text"
        );
        assert_eq!(
            contents_of_v, expected,
            "Contents of 2nd redirect target file should match input text"
        );
    }
}
