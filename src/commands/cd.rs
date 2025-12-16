use std::env;

use home;

pub const USAGE: &str = "Usage:\tcd [DIRECTORY]";

pub fn cd(input: &[String]) -> Result<String, String> {
    is_input_len_at_least_two(input)?;

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

fn is_input_len_at_least_two(input: &[String]) -> Result<(), String> {
    if input.len() > 2 {
        return Err(format!("Too many arguments\n{}", USAGE));
    }
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
        fs::create_dir(dest_path).expect("failed to create temp folder");
        let origin = env::current_dir().expect("failed to get current directory");

        let mut input = string_vec!["cd", destination];
        assert!(cd(&input).is_ok(), "`cd {}` should be ok", destination);

        input = string_vec!["cd", ".."];
        assert!(cd(&input).is_ok(), "`cd ..` should be ok");

        let current_dir = env::current_dir().expect("failed to get current directory");
        assert_eq!(origin, current_dir);
    }
}
