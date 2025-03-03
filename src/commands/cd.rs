use std::env;

use home;

pub fn cd(input: &[String]) -> Result<String, String> {
    debug_assert!(!input.is_empty(), "Input for `cd` should not be empty");
    debug_assert!(
        input[0] == "cd",
        "Input for `{}` should not be passed to `cd`",
        input[0]
    );

    if input.len() > 2 {
        return Err("Too many arguments".to_string());
    }

    let path: &String = match input.get(1) {
        Some(path) => path,
        None => {
            if let Some(home_path) = home::home_dir() {
                env::set_current_dir(&home_path).map_err(|err| {
                    format!("Failed to change directory: {}", err)
                        .split(" (os ")
                        .next()
                        .unwrap_or(" ")
                        .to_string()
                })?;
                return Ok(String::new());
            } else {
                return Err("Could not determine home directory".to_string());
            }
        }
    };

    match env::set_current_dir(path) {
        Ok(_) => Ok(String::new()),
        Err(err) => Err(format!("{}: {}", path, err)
            .split(" (os ")
            .next()
            .unwrap_or(" ")
            .to_string()),
    }
}

#[cfg(test)]
mod test {
    use super::cd;
    use crate::string_vec;

    #[test]
    fn test_cd() {
        let mut input = string_vec!["cd", ".."];
        assert!(cd(&input).is_ok(), "`cd ..` should be ok");

        input = string_vec!["cd", "0-shell"];
        assert!(cd(&input).is_ok(), "`cd 0-shell` should be ok");
    }
}
