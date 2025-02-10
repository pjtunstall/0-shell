#![allow(dead_code)]
use std::env;
use std::io::{self, Write};

fn main() {
    loop {
        let input = get_input().unwrap();
        let input = split(&input);
        let command = input[0].as_str();

        match command {
            "pwd" => match pwd(&input) {
                Ok(ok) => println!("{}", ok),
                Err(err) => handle_error(command, err),
            },
            _ => {
                continue;
            }
        }
    }
}

fn handle_error(command: &str, err: String) {
    eprintln!("{}: {}", command, err.to_lowercase());
}

fn pwd(input: &Vec<String>) -> Result<String, String> {
    if input.len() > 1 {
        return Err("too many arguments".to_string());
    }
    let cwd = match get_current_dir() {
        Ok(cwd) => cwd,
        Err(err) => return Err(format!("getcwd: {}", err)),
    };
    let ok = format!("{}", cwd);
    Ok(ok)
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
    io::stdout().flush()?; // Ensure the prompt is printed before waiting for input. In some cases, the prompt may not show immediately.

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}

#[cfg(test)]
mod tests {
    use std::path::MAIN_SEPARATOR;

    use super::*;

    fn test_pwd_success() {
        let input = "pwd";
        let expected = "shell";
        let result = pwd(&split(input)).unwrap();
        let last_segment = result.split(MAIN_SEPARATOR).last().unwrap();
        assert_eq!(last_segment, expected);
    }

    #[test]
    fn test_pwd_too_many_args() {
        let input = "pwd foo";
        let expected = Err("too many arguments".to_string());
        assert_eq!(pwd(&split(input)), expected);
    }
}
