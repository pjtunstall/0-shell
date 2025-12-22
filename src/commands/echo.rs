use std::{env, fs::OpenOptions, io::Write};

use serde_json::de::from_str;

use crate::redirect;

pub const USAGE: &str = "Usage:\techo [STRING]...";

pub fn echo(input: &[String]) -> Result<String, String> {
    let (sources, targets) = redirect::separate_sources_from_targets(input);

    if input.len() < 2 {
        return Ok(String::from("\n"));
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
            .map(String::from)
            .unwrap_or_else(|| String::from(" "))
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

    // In case the string ends with backslashes.
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

    use super::echo;
    use crate::{string_vec, test_helpers::TempStore};

    #[test]
    fn echo_basic() {
        assert_eq!(
            echo(&string_vec!["echo", "hello"]),
            Ok(String::from("hello\n")),
            "Expected to echo one word"
        );
        assert_eq!(
            echo(&string_vec!["echo", "hello", "world"]),
            Ok(String::from("hello world\n")),
            "Expected to echo two words"
        );
        assert_eq!(
            echo(&string_vec!["echo", "hello", "world", "hello"]),
            Ok(String::from("hello world hello\n")),
            "Expected to echo three words"
        );
    }

    #[test]
    fn echo_special_characters() {
        assert_eq!(
            echo(&string_vec!["echo", "a\\na"]),
            Ok(String::from("ana\n")),
            "Expected to convert `\\n` to `n`"
        );
        assert_eq!(
            echo(&string_vec!["echo", "a\\\\na"]),
            Ok(String::from("a\na\n")),
            "Expected to convert `\\\\n` to `\\n`"
        );
        assert_eq!(
            echo(&string_vec!["echo", "a\\\\\\na"]),
            Ok(String::from("a\na\n")),
            "Expected to convert `\\\\\\n` to `\\n`"
        );
        assert_eq!(
            echo(&string_vec!["echo", "a\\\\\\\\na"]),
            Ok(String::from("a\\na\n")),
            "Expected to convert `\\\\\\\\n` to `\\\\n`"
        );

        assert_eq!(
            echo(&string_vec!["echo", "\"a\\na\""]),
            Ok(String::from("a\na\n")),
            "Expected to leave `\\n` unchanged in quotes"
        );
        assert_eq!(
            echo(&string_vec![
                String::from("echo"),
                String::from("\"a\\\\na\"")
            ]),
            Ok(String::from("a\na\n")),
            "Expected to leave `\\\\n` unchanged in quotes"
        );
        assert_eq!(
            echo(&string_vec!["echo", "\"a\\\\\\na\""]),
            Ok(String::from("a\\na\n")),
            "Expected to convert `\\\\\\n` in quotes to `\\n`"
        );
        assert_eq!(
            echo(&string_vec!["echo", "\"a\\\\\\\\na\""]),
            Ok(String::from("a\\na\n")),
            "Expected to convert `\\\\\\\\n` in quotes to `\\n`"
        );
    }

    #[test]
    fn echo_redirection_in_double_quotes() {
        assert_eq!(
            echo(&string_vec!["echo", "\">\""]),
            Ok(String::from(">\n")),
            "Expected to escape `>` in double quotes, and give no error when final"
        );

        assert_eq!(
            echo(&string_vec!["echo", "\">>\""]),
            Ok(String::from(">>\n")),
            "Expected to leave `>>` unchanged in double quotes, and give no error when final"
        );
    }

    #[test]
    fn echo_redirection_in_single_quotes() {
        assert_eq!(
            echo(&string_vec!["echo", "\'>\'"]),
            Ok(String::from(">\n")),
            "Expected to leave `>` unchanged in single quotes, and give no error when final"
        );

        assert_eq!(
            echo(&string_vec!["echo", "\'>>\'"]),
            Ok(String::from(">>\n")),
            "Expected to leave `>>` unchanged in single quotes, and give no error when final"
        );
    }

    #[test]
    fn echo_env_var_set() {
        let prev_user = env::var("USER").ok();
        unsafe {
            env::set_var("USER", "testuser");
        }

        assert_eq!(
            echo(&string_vec!["echo", "$USER"]),
            Ok(String::from("testuser\n")),
            "expected `USER` to be replaced with `testuser`"
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
    fn echo_env_var_unset() {
        let prev_lang = env::var("LANG").ok();
        unsafe {
            env::remove_var("LANG");
        }
        assert_eq!(
            echo(&string_vec!["echo", "$LANG"]),
            Ok(String::from("\n")),
            "expected empty substitution when LANG is unset"
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
    fn echo_redirect_write() {
        let file = &TempStore::new(1).store[0];

        let mut expected = "hello\n";
        let mut input = string_vec!["echo", "hello", ">", file];
        let mut output = echo(&input);
        assert!(output.unwrap().is_empty(), "output should be empty");
        let mut contents = fs::read_to_string(&file).expect("failed to read file");
        assert_eq!(contents, expected, "Expected to write to nonexistent file");

        expected = "world\n";
        input = string_vec!["echo", "world", ">", file];
        output = echo(&input);
        assert!(output.unwrap().is_empty(), "output should be empty");
        contents = fs::read_to_string(&file).expect("failed to read file");
        assert_eq!(contents, expected, "Expected to overwrite existing file");
    }

    #[test]
    fn echo_redirect_append() {
        let file = &TempStore::new(1).store[0];

        let mut input = string_vec!["echo", "hello", ">>", file];
        let mut expected = "hello\n";
        let mut output = echo(&input);
        assert!(output.is_ok());
        let mut contents = fs::read_to_string(&file).expect("failed to read file");
        assert_eq!(contents, expected, "Expected to append to nonexistent file");

        input = string_vec!["echo", "world", ">>", file];
        expected = "hello\nworld\n";
        output = echo(&input);
        assert!(output.unwrap().is_empty(), "output should be empty");
        contents = fs::read_to_string(&file).expect("failed to read file");
        assert_eq!(contents, expected, "Expected to append to existing file");
    }

    #[test]
    fn echo_redirect_write_interpolated() {
        let file1 = &TempStore::new(1).store[0];

        let input = string_vec!["echo", "hello", ">", file1, "file2"];
        let expected = "hello file2\n";
        let output = echo(&input);

        assert!(output.unwrap().is_empty(), "output should be empty");
        let contents = fs::read_to_string(&file1).expect("failed to read file");
        assert!(
            !Path::new("file2").exists(),
            "expected to write to only one file when two names appear after a single write operator"
        );
        assert_eq!(contents, expected, "contents should include `file2`");
    }

    #[test]
    fn echo_redirect_append_interpolated() {
        let file1 = &TempStore::new(1).store[0];

        let input = string_vec!["echo", "hello", ">>", file1, "file2"];
        let expected = "hello file2\n";
        let output = echo(&input);
        assert!(output.unwrap().is_empty(), "output should be empty");
        let contents = fs::read_to_string(&file1).expect("failed to read file");
        assert!(
            !Path::new("file2").exists(),
            "expected to write to only one file when two names appear after a single append operator"
        );
        assert_eq!(contents, expected, "contents should include `file2`");
    }

    #[test]
    fn echo_multiple_redirect_targets() {
        let temp_store = TempStore::new(2);
        let u_str = &temp_store.store[0];
        let v_str = &temp_store.store[1];

        let input: Vec<String> = string_vec!["echo", "hello", ">", u_str, ">", v_str];

        let result = echo(&input);
        assert!(result.is_ok(), "result of multiple redirects should be ok");

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

        let contents_of_u = fs::read_to_string(u).expect("failed to read 1st redirect target file");
        let contents_of_v = fs::read_to_string(v).expect("failed to read 2nd redierct target file");

        let expected = "hello\n";

        assert_eq!(
            contents_of_u, expected,
            "contents of 1st redirect target file should match input text"
        );
        assert_eq!(
            contents_of_v, expected,
            "contents of 2nd redirect target file should match input text"
        );
    }

    #[test]
    fn echo_redirect_to_directory_fails() {
        let temp_store = TempStore::new(2);
        let dir = &temp_store.store[0];
        let file = &temp_store.store[1];

        fs::create_dir(dir).expect("failed to create target directory");

        let input = string_vec!["echo", "hello", ">", dir];
        let result = echo(&input);
        assert!(result.is_err(), "redirect to directory should fail");

        // Ensure no stray file gets created alongside the directory.
        assert!(!Path::new(file).exists(), "no extra file should be created");
        assert!(
            result
                .unwrap_err()
                .to_lowercase()
                .contains("is a directory"),
            "error should mention directory"
        );
    }
}
