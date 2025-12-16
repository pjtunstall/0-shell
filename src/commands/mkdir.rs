use std::{fs, path::Path};

pub const USAGE: &str = "Usage:\tmkdir DIRECTORY...";

pub fn mkdir(input: &[String]) -> Result<String, String> {
    is_input_len_at_least_two(input)?;

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

fn is_input_len_at_least_two(input: &[String]) -> Result<(), String> {
    if input.len() < 2 {
        return Err(format!("Not enough arguments\n{}", USAGE));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use uuid::Uuid;

    use super::{USAGE, mkdir};
    use crate::{string_vec, test_helpers::TempStore};

    #[test]
    fn mkdir_ok() {
        let test_dir = &Uuid::new_v4().to_string();

        if Path::new(test_dir).exists() {
            fs::remove_dir(test_dir).expect("failed to remove directory");
        }

        let input = string_vec!["mkdir", test_dir];
        let result = mkdir(&input);

        assert!(result.is_ok());
        assert!(Path::new(test_dir).exists());

        fs::remove_dir(test_dir).expect("failed to remove directory");
    }

    #[test]
    fn mkdir_missing_argument_fails() {
        let input = string_vec!["mkdir"];
        let result = mkdir(&input);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            format!("Not enough arguments\n{}", USAGE)
        );
    }

    #[test]
    fn mkdir_invalid_path_fails() {
        let temp_store = TempStore::new(2);
        let dir = Path::new(&temp_store.store[0]);
        let prefix = Path::new(&temp_store.store[1]);
        let invalid_path = prefix.join(dir);
        let invalid_str = invalid_path
            .to_str()
            .expect("failed to get string from invalid path");

        let input = string_vec!["mkdir", invalid_str];
        let result = mkdir(&input);

        assert!(result.is_err());
    }

    #[test]
    fn mkdir_dir_exists_fails() {
        let temp_store = TempStore::new(1);
        let dir = temp_store.store[0].clone();

        fs::create_dir(Path::new(&dir)).expect("failed to create test directory");

        let input = string_vec!["mkdir", dir];
        let result = mkdir(&input);

        assert!(result.is_err());
    }

    #[test]
    fn mkdir_file_exists_fails() {
        let temp_store = TempStore::new(1);
        let dir = temp_store.store[0].clone();

        fs::write(Path::new(&dir), "").expect("failed to create test file");

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

        fs::create_dir(new_path).expect("failed to create test directory");

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
