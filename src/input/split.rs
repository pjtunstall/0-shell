pub fn split(input: &str) -> Result<Vec<String>, String> {
    if let Some((part_1, divider, part_2)) = split_at_first_divider(input) {
        if part_2.is_empty() {
            return Err(String::from("Parse error near `\\n'"));
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
                        quote_char = None; // Closing matching quote.
                    }
                } else {
                    quote_char = Some(c); // Opening new quote.
                }
            }
            '>' if quote_char.is_none() => {
                if let Some((_, '>')) = chars.peek() {
                    return Some((
                        String::from(&input[..i]),
                        String::from(">>"),
                        String::from(input.get(i + 2..).unwrap_or("")),
                    ));
                } else {
                    return Some((
                        String::from(&input[..i]),
                        String::from(">"),
                        String::from(input.get(i + 1..).unwrap_or("")),
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
                // Closing quote: add the word to the result and reset.
                inside_quotes = false;
                current_word.push(c); // Add the closing quote.
                result.push(current_word);
                current_word = String::new();
            } else {
                // Add character to current word inside quotes.
                current_word.push(c);
            }
        } else {
            if c == '>' {
                return Err(String::from("parse error near `\\n'"));
            }
            if c == '"' || c == '\'' {
                // Opening quote: start capturing the word inside quotes.
                if !current_word.is_empty() {
                    result.push(current_word);
                    current_word = String::new();
                }
                inside_quotes = true;
                current_word.push(c); // Add the opening quote.
            } else if c.is_whitespace() {
                // Split on whitespace when outside of quotes.
                if !current_word.is_empty() {
                    result.push(current_word);
                    current_word = String::new();
                }
            } else {
                // Add non-whitespace characters to the current word.
                current_word.push(c);
            }
        }
    }

    // Push the last word if exists.
    if !current_word.is_empty() {
        result.push(current_word);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::split;
    use crate::string_vec;

    #[test]
    fn test_split() {
        assert_eq!(split(""), Ok(vec![]));
        assert_eq!(
            split("echo foo bar>>baz > qux"),
            Ok(string_vec!["echo", "foo", "bar", ">>", "baz", ">", "qux"])
        );
    }
}
