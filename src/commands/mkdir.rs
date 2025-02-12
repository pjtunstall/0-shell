use std::fs;

pub fn mkdir(input: &Vec<String>) -> Result<String, String> {
    if let Err(err) = crate::helpers::check_num_args(input, 2) {
        return Err(err);
    }

    let path = input.get(1).ok_or_else(|| "missing argument".to_string())?;

    fs::create_dir(path).map_err(|err| err.to_string())?;

    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use super::mkdir;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_mkdir_success() {
        let test_dir = "test_dir";

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
        let invalid_path = "/invalid/path/to/dir";
        let input = vec!["mkdir".to_string(), invalid_path.to_string()];
        let result = mkdir(&input);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "no such file or directory");
    }
}
