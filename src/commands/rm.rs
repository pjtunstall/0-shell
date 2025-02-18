use std::{fs, path::Path};

pub fn rm(input: &Vec<String>) -> Result<String, String> {
    debug_assert!(!input.is_empty(), "Input for `rm` should not be empty");
    debug_assert!(
        input[0] == "rm",
        "Input for `{}` should not be passed to `rm`",
        input[0]
    );

    let path = Path::new(&input[1]);

    if path.is_dir() {
        return Err(format!("{}: is a directory", &input[1]));
    }

    if let Err(err) = fs::remove_file(path) {
        return Err(format!("{}: {}", &input[1], err));
    }

    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::TempStore;

    #[test]
    fn test_rm() {
        let temp_store = TempStore::new();
        let file = &temp_store.target;
        fs::write(file.to_string(), "").expect("Failed to create test file");

        let input = vec!["rm".to_string(), file.to_string()];
        let result = rm(&input);

        assert!(result.is_ok());
        assert!(!Path::new(file).exists(), "File should have been removed");
    }

    #[test]
    fn test_rm_error_when_argument_is_directory() {
        let temp_store = TempStore::new();
        let dir = &temp_store.target;
        fs::create_dir(dir.to_string()).expect("Failed to create test directory");

        let input = vec!["rm".to_string(), dir.to_string()];
        let result = rm(&input);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), format!("{}: is a directory", dir));
        assert!(
            Path::new(dir).exists(),
            "Directory should not have been removed"
        );
    }
}
