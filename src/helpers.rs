use std::{env, io};

pub fn check_num_args(input: &Vec<String>, expected: usize) -> Result<String, String> {
    if input.len() > expected {
        return Err("too many arguments".to_string());
    } else if input.len() < expected {
        return Err("missing argument".to_string());
    }
    Ok(String::new())
}

pub fn get_current_dir() -> io::Result<String> {
    let cwd = env::current_dir()?;
    let cwd = format!("{}", cwd.display());
    Ok(cwd)
}

pub fn split(input: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current_word = String::new();
    let mut inside_quotes = false;

    for c in input.chars() {
        if inside_quotes {
            if c == '"' {
                // Closing quote, add the word to the result and reset
                inside_quotes = false;
                current_word.push(c); // Add the closing quote
                result.push(current_word);
                current_word = String::new();
            } else {
                // Add character to current word inside quotes
                current_word.push(c);
            }
        } else {
            if c == '"' {
                // Opening quote, start capturing the word inside quotes
                if !current_word.is_empty() {
                    result.push(current_word);
                    current_word = String::new();
                }
                inside_quotes = true;
                current_word.push(c); // Add the opening quote
            } else if c.is_whitespace() {
                // Split on whitespace when outside of quotes
                if !current_word.is_empty() {
                    result.push(current_word);
                    current_word = String::new();
                }
            } else {
                // Add non-whitespace characters to the current word
                current_word.push(c);
            }
        }
    }

    // Push the last word if exists
    if !current_word.is_empty() {
        result.push(current_word);
    }

    result
}
