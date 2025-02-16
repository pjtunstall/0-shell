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

pub fn split(input: &str) -> Result<Vec<String>, String> {
    if let Some((part_1, divider, part_2)) = split_at_first_divider(input) {
        if part_2.is_empty() {
            return Err("parse error near `\\n'".to_string());
        }

        let mut result = Vec::new();

        let part_1 = split(&part_1)?;
        let part_2 = split(&part_2)?;

        result.extend(part_1);
        result.push(divider);
        result.extend(part_2);

        Ok(result)
    } else {
        Ok(split_part(input))
    }
}

fn split_at_first_divider(input: &str) -> Option<(String, String, String)> {
    let mut in_quotes = false;
    let mut quote_char = '\0';

    let mut chars = input.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        match c {
            '"' | '\'' => {
                if in_quotes && c == quote_char {
                    in_quotes = false; // Closing quote
                } else if !in_quotes {
                    in_quotes = true;
                    quote_char = c; // Track which quote type
                }
            }
            '>' if !in_quotes => {
                // Check if it's ">>"
                if let Some((_, '>')) = chars.peek() {
                    return Some((
                        input[..i].to_string(),
                        ">>".to_string(),
                        input[i + 2..].to_string(),
                    ));
                } else {
                    return Some((
                        input[..i].to_string(),
                        ">".to_string(),
                        input[i + 1..].to_string(),
                    ));
                }
            }
            _ => {}
        }
    }
    None
}

fn split_part(input: &str) -> Vec<String> {
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
            if c == '>' {
                println!("parse error near `\\n'");
                return Vec::new();
            }
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
