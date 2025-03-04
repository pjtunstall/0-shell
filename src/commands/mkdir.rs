use std::{fs, path::Path};

pub fn mkdir(input: &[String]) -> Result<String, String> {
    validate_input(input)?;

    let mut errors = Vec::new();

    for path_str in input[1..].iter() {
        let path = Path::new(path_str);

        if path.exists() {
            errors.push(format!("mkdir: {}: File exists", path.display()));
        } else {
            if let Err(err) = fs::create_dir(path) {
                errors.push(
                    err.to_string()
                        .split(" (os ")
                        .next()
                        .unwrap_or(" ")
                        .to_string(),
                );
            }
        }
    }

    if errors.is_empty() {
        Ok(String::new())
    } else {
        let joined_errors = errors.join("\n");
        if let Some(suffix) = joined_errors.strip_prefix("mkdir: ") {
            Err(suffix.to_string())
        } else {
            Err(joined_errors)
        }
    }
}

fn validate_input(input: &[String]) -> Result<(), String> {
    debug_assert!(!input.is_empty(), "Input for `mkdir` should not be empty");
    debug_assert!(
        input[0] == "mkdir",
        "Input for `{}` should not be passed to `mkdir`",
        input[0]
    );

    if input.len() < 2 {
        return Err("Not enough arguments".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use uuid::Uuid;

    use super::mkdir;
    use crate::{string_vec, test_helpers::TempStore};

    #[test]
    fn mkdir_ok() {
        let test_dir = &Uuid::new_v4().to_string();

        if Path::new(test_dir).exists() {
            fs::remove_dir(test_dir).unwrap();
        }

        let input = string_vec!["mkdir", test_dir];
        let result = mkdir(&input);

        assert!(result.is_ok());
        assert!(Path::new(test_dir).exists());

        fs::remove_dir(test_dir).unwrap();
    }

    #[test]
    fn mkdir_missing_argument_fails() {
        let input = string_vec!["mkdir"];
        let result = mkdir(&input);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not enough arguments");
    }

    #[test]
    fn mkdir_invalid_path_fails() {
        let temp_store = TempStore::new(2);
        let dir = Path::new(&temp_store.store[0]);
        let prefix = Path::new(&temp_store.store[1]);
        let invalid_path = prefix.join(dir);
        let invalid_str = invalid_path
            .to_str()
            .expect("Failed to get string from invalid path");

        let input = string_vec!["mkdir", invalid_str];
        let result = mkdir(&input);

        assert!(result.is_err());
    }

    #[test]
    fn mkdir_dir_exists_fails() {
        let temp_store = TempStore::new(1);
        let dir = temp_store.store[0].clone();

        fs::create_dir(Path::new(&dir)).expect("Failed to create test directory");

        let input = string_vec!["mkdir", dir];
        let result = mkdir(&input);

        assert!(result.is_err());
    }

    #[test]
    fn mkdir_file_exists_fails() {
        let temp_store = TempStore::new(1);
        let dir = temp_store.store[0].clone();

        fs::write(Path::new(&dir), "").expect("Failed to create test file");

        let input = string_vec!["mkdir", dir];
        let result = mkdir(&input);

        assert!(result.is_err());
    }

    #[test]
    fn mkdir_multiple_arguments() {
        let temp_store = TempStore::new(2);

        let existing_string = &temp_store.store[0];
        let new_string = &temp_store.store[1];

        let new_path = Path::new(new_string);

        fs::create_dir(new_path).expect("Failed to create test directory");

        let input = string_vec!["mkdir", existing_string, new_string];
        let result = mkdir(&input);

        assert!(
            result.is_err(),
            "Result should be an error because one of the arguments already exists"
        );
        assert!(new_path.exists(), "New path should exist");
        assert!(new_path.is_dir(), "New path should be a directory");
    }
}
