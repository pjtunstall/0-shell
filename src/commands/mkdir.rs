use std::fs;

pub fn mkdir(input: &Vec<String>) -> Result<String, String> {
    debug_assert!(!input.is_empty(), "Input for `mkdir` should not be empty");
    debug_assert!(
        input[0] == "mkdir",
        "Input for `{}` should not be passed to `mkdir`",
        input[0]
    );

    if let Err(err) = crate::helpers::check_num_args(input, 2) {
        return Err(err);
    }

    let path = input.get(1).ok_or_else(|| "missing argument".to_string())?;

    fs::create_dir(path).map_err(|err| err.to_string().to_lowercase())?;

    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use uuid::Uuid;

    use super::mkdir;
    use crate::test_helpers::TempStore;

    #[test]
    fn test_mkdir_success() {
        let test_dir = &Uuid::new_v4().to_string();

        if Path::new(test_dir).exists() {
            fs::remove_dir(test_dir).unwrap();
        }

        let input = vec!["mkdir".to_string(), test_dir.to_string()];
        let result = mkdir(&input);

        assert!(result.is_ok());
        assert!(Path::new(test_dir).exists());

        fs::remove_dir(test_dir).unwrap();
    }

    #[test]
    fn test_mkdir_missing_argument() {
        let input = vec!["mkdir".to_string()];
        let result = mkdir(&input);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "missing argument");
    }

    #[test]
    fn test_mkdir_invalid_args() {
        let input = vec!["mkdir".to_string(), "dir".to_string(), "extra".to_string()];
        let result = mkdir(&input);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "too many arguments");
    }

    #[test]
    fn test_mkdir_invalid_path() {
        let temp_store = TempStore::new(2);
        let dir = Path::new(&temp_store.store[0]);
        let prefix = Path::new(&temp_store.store[1]);
        let invalid_path = prefix.join(dir);

        let input = vec![
            "mkdir".to_string(),
            invalid_path.to_str().unwrap().to_string(),
        ];
        let result = mkdir(&input);

        assert!(result.is_err());
    }
}
