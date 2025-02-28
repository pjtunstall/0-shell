use std::fs;

pub fn cat(input: &[String]) -> Result<String, String> {
    debug_assert!(!input.is_empty(), "Input for `cat` should not be empty");
    debug_assert!(
        input[0] == "cat",
        "Input for `{}` should not be passed to `cat`",
        input[0]
    );

    let filename = match input.get(1) {
        Some(filename) => filename,
        None => return Err("Not enough arguments".to_string()),
    };
    match fs::read_to_string(filename) {
        Ok(contents) => Ok(format!("{}", contents)),
        Err(err) => {
            return Err(format!("Error reading file: {}", err)
                .split(" (os ")
                .next()
                .unwrap_or(" ")
                .to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use uuid::Uuid;

    use super::cat;

    #[test]
    fn test_cat() {
        let file = Uuid::new_v4().to_string();
        fs::write(&file, "Howdie, world!\n").unwrap();

        let result = cat(&vec!["cat".to_string(), file.clone()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Howdie, world!\n".to_string());

        fs::remove_file(file).unwrap();
    }
}
