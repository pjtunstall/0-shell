use std::env;

use home;

pub fn cd(input: &[String]) -> Result<String, String> {
    validate_input(input)?;

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

fn validate_input(input: &[String]) -> Result<(), String> {
    debug_assert!(!input.is_empty(), "Input for `cd` should not be empty");
    debug_assert!(
        input[0] == "cd",
        "Input for `{}` should not be passed to `cd`",
        input[0]
    );
    Ok(())
}

#[cfg(test)]
mod test {
    use std::{env, fs, path::Path};

    use super::cd;
    use crate::string_vec;
    use crate::test_helpers;

    #[test]
    fn cd_and_return() {
        let temp_store = test_helpers::TempStore::new(1);
        let destination = &temp_store.store[0];
        let dest_path = Path::new(destination);
        fs::create_dir(dest_path).expect("Failed to create temp folder");
        let origin = env::current_dir().expect("Failed to get current directory");

        let mut input = string_vec!["cd", destination];
        assert!(cd(&input).is_ok(), "`cd {}` should be ok", destination);

        input = string_vec!["cd", ".."];
        assert!(cd(&input).is_ok(), "`cd ..` should be ok");

        let current_dir = env::current_dir().expect("Failed to get current directory");
        assert_eq!(origin, current_dir);
    }
}
