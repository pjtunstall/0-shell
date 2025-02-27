use std::{env, io};

pub fn check_num_args(input: &[String], expected: usize) -> Result<String, String> {
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
        split_part(input)
    }
}

fn split_at_first_divider(input: &str) -> Option<(String, String, String)> {
    let mut quote_char: Option<char> = None;
    let mut chars = input.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        match c {
            '"' | '\'' => {
                if let Some(q) = quote_char {
                    if q == c {
                        quote_char = None; // Closing matching quote
                    }
                } else {
                    quote_char = Some(c); // Opening new quote
                }
            }
            '>' if quote_char.is_none() => {
                if let Some((_, '>')) = chars.peek() {
                    return Some((
                        input[..i].to_string(),
                        ">>".to_string(),
                        input.get(i + 2..).unwrap_or("").to_string(),
                    ));
                } else {
                    return Some((
                        input[..i].to_string(),
                        ">".to_string(),
                        input.get(i + 1..).unwrap_or("").to_string(),
                    ));
                }
            }
            _ => {}
        }
    }
    None
}

fn split_part(input: &str) -> Result<Vec<String>, String> {
    let mut result = Vec::new();
    let mut current_word = String::new();
    let mut inside_quotes = false;

    for c in input.chars() {
        if inside_quotes {
            if c == '"' || c == '\'' {
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
                return Err("parse error near `\\n'".to_string());
            }
            if c == '"' || c == '\'' {
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

    Ok(result)
}
