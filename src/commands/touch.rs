use std::fs::File;
use std::path::Path;

pub fn touch(input: &[String]) -> Result<String, String> {
    debug_assert!(!input.is_empty(), "Input for `touch` should not be empty");
    debug_assert!(
        input[0] == "touch",
        "Input for `{}` should not be passed to `touch`",
        input[0]
    );

    if input.len() < 2 {
        return Err("Not enough arguments".to_string());
    }

    let mut errors = Vec::new();

    for path_str in input[1..].iter() {
        let path = Path::new(path_str);
        if path.exists() {
            if let Err(e) =
                filetime::set_file_times(path, filetime::FileTime::now(), filetime::FileTime::now())
            {
                errors.push(format!(
                    "{}: {}: {}",
                    "touch",
                    path_str,
                    e.to_string()
                        .split(" (os ")
                        .next()
                        .unwrap_or(" ")
                        .to_string()
                ));
            }
        } else {
            if let Err(e) = File::create(path) {
                errors.push(format!(
                    "{}: {}: {}",
                    "touch",
                    path_str,
                    e.to_string()
                        .split(" (os ")
                        .next()
                        .unwrap_or(" ")
                        .to_string()
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(String::new())
    } else {
        let joined_errors = errors.join("\n");
        if let Some(suffix) = joined_errors.strip_prefix("touch: ") {
            Err(suffix.to_string())
        } else {
            Err(joined_errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path, thread, time::Duration};

    use filetime::FileTime;

    use super::touch;
    use crate::test_helpers::TempStore;

    #[test]
    fn test_touch_success() {
        let temp_store = TempStore::new(1);
        let source = &temp_store.store[0];
        let path = Path::new(source);

        let input = vec![String::from("touch"), source.to_string()];
        let result = touch(&input);
        assert!(result.is_ok(), "Result should be ok");
        assert!(path.exists(), "New file should exist");
    }

    #[test]
    fn test_touch_failure_missing_argument() {
        let input = vec!["touch".to_string()];
        let result = touch(&input);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not enough arguments");
    }

    #[test]
    fn test_touch_failure_no_such_dir() {
        let temp_store = TempStore::new(2);
        let dir = Path::new(&temp_store.store[0]);
        let prefix = Path::new(&temp_store.store[1]);
        let invalid_path = prefix.join(dir);

        let input = vec![
            "touch".to_string(),
            invalid_path.to_str().unwrap().to_string(),
        ];
        let result = touch(&input);

        assert!(result.is_err());
    }

    #[test]
    fn test_touch_updates_time_of_existing_file() {
        let temp_store = TempStore::new(1);
        let file_string = &temp_store.store[0];

        let file_path = Path::new(file_string);
        fs::write(file_path, "").expect("Failed to create test file");

        let initial_metadata = fs::metadata(file_path).expect("Failed to get initial metadata");
        let initial_time = FileTime::from_last_modification_time(&initial_metadata);

        thread::sleep(Duration::from_millis(1024));

        let result = touch(&vec!["touch".to_string(), file_string.clone()]);
        assert!(result.is_ok());

        let updated_metadata = fs::metadata(file_path).expect("Failed to get final metadata");
        let updated_time = FileTime::from_last_modification_time(&updated_metadata);

        assert!(updated_time > initial_time);
    }
}
