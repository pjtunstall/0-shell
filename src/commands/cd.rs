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
        return Err("too many arguments".to_string());
    }

    let path: &String = match input.get(1) {
        Some(path) => path,
        None => {
            if let Some(home_path) = home::home_dir() {
                env::set_current_dir(&home_path)
                    .map_err(|e| format!("failed to change directory: {}", e))?;
                return Ok(String::new());
            } else {
                return Err("could not determine home directory".to_string());
            }
        }
    };

    match env::set_current_dir(path) {
        Ok(_) => Ok(String::new()),
        Err(e) => Err(format!("{}: {}", path, e)),
    }
}

#[cfg(test)]
mod test {
    use super::cd;

    #[test]
    fn test_cd() {
        assert!(
            cd(&vec!["cd".to_string(), "..".to_string()]).is_ok(),
            "`cd ..` should be ok"
        );
        assert!(
            cd(&vec!["cd".to_string(), "0-shell".to_string()]).is_ok(),
            "`cd 0-shell` should be ok"
        );
    }
}
