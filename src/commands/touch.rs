use std::{fs::File, path::Path};

pub const USAGE: &str = "Usage:\ttouch FILE...";

pub fn touch(input: &[String]) -> Result<String, String> {
    validate_input(input)?;

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

fn validate_input(input: &[String]) -> Result<(), String> {
    if input.len() < 2 {
        return Err(format!("Not enough arguments\n{}", USAGE));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path, thread, time::Duration};

    use filetime::FileTime;

    use super::{USAGE, touch};
    use crate::{string_vec, test_helpers::TempStore};

    #[test]
    fn touch_new_file() {
        let temp_store = TempStore::new(1);
        let source = &temp_store.store[0];
        let path = Path::new(source);

        let input = string_vec!["touch", source];
        let result = touch(&input);
        assert!(result.is_ok(), "Result should be ok");
        assert!(path.exists(), "New file should exist");
    }

    #[test]
    fn touch_insufficient_arguments_fails() {
        let input = string_vec!["touch"];
        let result = touch(&input);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            format!("Not enough arguments\n{}", USAGE)
        );
    }

    #[test]
    fn touch_invalid_path_fails() {
        let temp_store = TempStore::new(2);
        let dir = Path::new(&temp_store.store[0]);
        let prefix = Path::new(&temp_store.store[1]);
        let invalid_path = prefix.join(dir);
        let invalid_str = invalid_path
            .to_str()
            .expect("Failed to turn path to nonexistent directory back into &str");

        let input = string_vec!["touch", invalid_str];
        let result = touch(&input);

        assert!(result.is_err());
    }

    #[test]
    fn touch_multiple_arguments() {
        let temp_store = TempStore::new(3);
        let dir = Path::new(&temp_store.store[0]);
        let prefix = Path::new(&temp_store.store[1]);
        let invalid_path = prefix.join(dir);
        let valid_string = &temp_store.store[2];
        let invalid_string = format!("{}", invalid_path.display());

        let input = string_vec!["touch", valid_string, invalid_string];

        let result = touch(&input);

        assert!(
            result.is_err(),
            "Result should be an error because one of the paths is invalid"
        );
        assert!(Path::new(valid_string).exists(), "Valid path should exist");
    }

    #[test]
    fn touch_updates_time_of_existing_file() {
        let temp_store = TempStore::new(1);
        let file_string = &temp_store.store[0];

        let file_path = Path::new(file_string);
        fs::write(file_path, "").expect("Failed to create test file");

        let initial_metadata = fs::metadata(file_path).expect("Failed to get initial metadata");
        let initial_time = FileTime::from_last_modification_time(&initial_metadata);

        thread::sleep(Duration::from_millis(1024));

        let input = string_vec!["touch", file_string];
        let result = touch(&input);
        assert!(result.is_ok());

        let updated_metadata = fs::metadata(file_path).expect("Failed to get final metadata");
        let updated_time = FileTime::from_last_modification_time(&updated_metadata);

        assert!(updated_time > initial_time);
    }
}
