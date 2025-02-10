#![allow(dead_code)]
use std::env;
use std::io::{self, Write};

fn main() {
    loop {
        let input = get_input().unwrap();
        let input = split(&input);
        let command = input[0].as_str();

        match command {
            "pwd" => {
                if let Err(msg) = pwd(&input) {
                    handle_error(command, msg);
                }
            }
            _ => {
                continue;
            }
        }
    }
}

fn handle_error(command: &str, msg: String) {
    eprintln!("{}: {}", command, msg.to_lowercase());
}

fn pwd(input: &Vec<String>) -> Result<(), String> {
    if input.len() > 1 {
        return Err("too many arguments".to_string());
    }
    let cwd = match get_current_dir() {
        Ok(cwd) => cwd,
        Err(err) => return Err(format!("getcwd: {}", err)),
    };
    println!("{}", cwd);
    Ok(())
}

fn get_current_dir() -> io::Result<String> {
    let cwd = env::current_dir()?;
    let cwd = format!("{}", cwd.display());
    Ok(cwd)
}

fn split(input: &str) -> Vec<String> {
    input.split_whitespace().map(|s| s.to_string()).collect()
}

fn get_input() -> io::Result<String> {
    let cwd = get_current_dir().unwrap();
    print!("{} $ ", cwd);
    io::stdout().flush()?; // Ensure the prompt is printed before waiting for input. in some cases where the prompt may not show immediately.

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pwd_success() {
        let input = vec![];
        let result = pwd(&input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pwd_too_many_arguments() {
        let input = vec!["extra".to_string()];
        let result = pwd(&input);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "too many arguments");
    }

    #[test]
    fn test_pwd_unexpected_error() {
        use std::env;
        use std::fs;
        use tempfile::tempdir;

        // Create a temporary directory
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        // Remove the directory to cause an error
        fs::remove_dir(&temp_path).unwrap();

        // Try changing into the deleted directory
        let original_dir = env::current_dir().unwrap();
        let result = env::set_current_dir(&temp_path);

        if result.is_err() {
            // If we already can't enter the directory, just test pwd directly
            let pwd_result = pwd(&vec![]);
            assert!(pwd_result.is_err());
            assert!(pwd_result.unwrap_err().starts_with("getcwd:"));
        } else {
            // Otherwise, call pwd and expect it to fail
            let pwd_result = pwd(&vec![]);
            assert!(pwd_result.is_err());
            assert!(pwd_result.unwrap_err().starts_with("getcwd:"));
        }

        // Restore the original working directory
        env::set_current_dir(original_dir).unwrap();
    }
}
