use std::{env, io};

pub fn check_num_args(input: &Vec<String>, expected: usize) -> Result<String, String> {
    if input.len() > expected {
        return Err("too many arguments".to_string());
    } else if input.len() < expected {
        return Err("missing argument".to_string());
    }
    Ok(String::new())
}

pub fn split(input: &str) -> Vec<String> {
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

pub fn get_current_dir() -> io::Result<String> {
    let cwd = env::current_dir()?;
    let cwd = format!("{}", cwd.display());
    Ok(cwd)
}
