use std::{
    env,
    io::{self, Write},
    process,
};

struct TextStyle;

impl Drop for TextStyle {
    fn drop(&mut self) {
        // Reset formatting to normal when the item is dropped
        print!("\x1b[0m");
    }
}

fn red_println(text: &str) {
    println!("\x1b[31m{}\x1b[0m\x1b[1m", text);
}

fn main() {
    print!("\x1b[1m");
    let _bold_text = TextStyle;

    loop {
        let prompt = prompt().unwrap_or_else(|err| {
            panic!("Failed to generate prompt: {}", err);
        });
        print!("{}", &prompt);
        io::stdout().flush().expect("Failed to flush stdout"); // When you use functions like print!, println!, or other write operations to stdout, the output is typically buffered. This means that the data doesn't immediately go to the terminal or file but is stored temporarily in memory until it's flushed (or until the buffer is full). For example, if you use println!, it automatically appends a newline, which generally flushes the output, but in some cases (such as with print!), you need to explicitly flush the output to ensure it’s immediately written to the terminal.

        let input = get_input().unwrap();
        let input = split(&input);
        if input.is_empty() {
            continue;
        }

        let command = input[0].as_str();

        let result = match command {
            "cd" => cd(&input),
            "echo" => echo(&input),
            "exit" => exit(&input),
            "pwd" => pwd(&input),
            _ => {
                handle_error(command, "command not found".to_string());
                continue;
            }
        };

        match result {
            Ok(ok) => {
                if !ok.is_empty() {
                    println!("{}", &ok);
                }
            }
            Err(err) => handle_error(command, err),
        }
    }
}

fn echo(input: &Vec<String>) -> Result<String, String> {
    if let Err(err) = check_num_args(input, 2) {
        return Err(err);
    }
    Ok(input[1].clone())
}

fn exit(input: &Vec<String>) -> Result<String, String> {
    if input.len() > 1 {
        return Err("too many arguments".to_string());
    }
    process::exit(0);
}

fn handle_error(command: &str, err: String) {
    red_println(&format!("{}: {}", command, err.to_lowercase()));
}

fn check_num_args(input: &Vec<String>, expected: usize) -> Result<String, String> {
    if input.len() > expected {
        return Err("too many arguments".to_string());
    } else if input.len() < expected {
        return Err("missing argument".to_string());
    }
    Ok(String::new())
}

fn pwd(input: &Vec<String>) -> Result<String, String> {
    if let Err(err) = check_num_args(input, 1) {
        return Err(err);
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
    input
        .split('"')
        .enumerate()
        .flat_map(|(i, part)| {
            if i % 2 == 0 {
                part.split_whitespace()
                    .map(String::from)
                    .collect::<Vec<_>>()
            } else {
                vec![part.to_string().replace(r"\r\n", "\n").replace(r"\n", "\n")]
            }
        })
        .collect()
}

fn prompt() -> io::Result<String> {
    let cwd = get_current_dir()?;
    let prompt = format!("{} $ ", cwd);
    Ok(prompt)
}

fn get_input() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}

fn cd(input: &Vec<String>) -> Result<String, String> {
    if let Err(err) = check_num_args(input, 2) {
        return Err(err);
    }

    let path = match input.get(1) {
        Some(path) => path,
        None => return Err("missing argument".to_string()),
    };

    match env::set_current_dir(path) {
        Ok(_) => Ok(String::new()),
        Err(e) => Err(format!("{}: {}", path, e)),
    }
}

#[cfg(test)]
mod tests {
    use std::path::MAIN_SEPARATOR;

    use super::*;

    #[test]
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
