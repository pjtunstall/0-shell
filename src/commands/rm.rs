use std::{fs, path::Path};

pub const USAGE: &str = "Usage:\trm [-r] <FILE|DIRECTORY>...";

pub const OPTIONS_USAGE: &str = "\r\n-R  -r  -- remove directories and their contents recursively";

pub fn rm(input: &[String]) -> Result<String, String> {
    is_input_len_at_least_two(input)?;

    if input[1] == "-r" || input[1] == "-R" {
        if input.len() < 3 {
            return Err(format!("Not enough arguments\n{}", USAGE));
        }

        return process_args(&input[2..], true);
    } else {
        return process_args(&input[1..], false);
    }
}

fn process_args(args: &[String], recursive: bool) -> Result<String, String> {
    let mut errors: Vec<Result<(), String>> = Vec::new();
    for arg in args.iter() {
        let path = Path::new(&arg);

        if recursive {
            if path.is_dir() {
                _ = fs::remove_dir_all(path).map_err(|err| {
                    errors.push(Err(format!(
                        "Failed to remove directory {}: {}",
                        arg,
                        err.to_string()
                            .split(" (os")
                            .next()
                            .unwrap_or(" ")
                            .to_string()
                    )));
                });
            } else {
                _ = fs::remove_file(path).map_err(|err| {
                    errors.push(Err(format!(
                        "Failed to remove file {}: {}",
                        arg,
                        err.to_string()
                            .split(" (os ")
                            .next()
                            .unwrap_or(" ")
                            .to_string()
                    )));
                });
            }
        } else {
            if path.is_dir() {
                errors.push(Err(format!("Is a directory: {}", arg)));
            } else if let Err(err) = fs::remove_file(path) {
                errors.push(Err(format!(
                    "Failed to remove file {}: {}",
                    arg,
                    err.to_string()
                        .split(" (os ")
                        .next()
                        .unwrap_or(" ")
                        .to_string()
                )));
            }
        }
    }

    if !errors.is_empty() {
        let error_messages = errors
            .into_iter()
            .filter_map(|e| e.err())
            .collect::<Vec<String>>()
            .join("\n");
        return Err(error_messages);
    }

    return Ok(String::new());
}

fn is_input_len_at_least_two(input: &[String]) -> Result<(), String> {
    if input.len() < 2 {
        return Err(format!("Not enough arguments\n{}", USAGE).to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use super::rm;
    use crate::{string_vec, test_helpers::TempStore};

    #[test]
    fn rm_removes_one_file() {
        let temp_store = TempStore::new(1);
        let file = &temp_store.store[0];
        fs::write(file.to_string(), "").expect("failed to create test file");

        let input = string_vec!["rm", file];
        let result = rm(&input);

        assert!(result.is_ok());
        assert!(!Path::new(file).exists(), "File should have been removed");
    }

    #[test]
    fn rm_removes_multiple_files() {
        let temp_store = TempStore::new(2);
        let file1 = &temp_store.store[0];
        let file2 = &temp_store.store[1];
        fs::write(file1.to_string(), "").expect("failed to create test file");
        fs::write(file2.to_string(), "").expect("failed to create test file");

        let input = string_vec!["rm", file1, file2];
        let result = rm(&input);

        assert!(result.is_ok());
        assert!(!Path::new(file1).exists(), "File should have been removed");
        assert!(!Path::new(file2).exists(), "File should have been removed");
    }

    #[test]
    fn rm_directory_fails() {
        let temp_store = TempStore::new(1);
        let dir = &temp_store.store[0];
        fs::create_dir(dir.to_string()).expect("failed to create test directory");

        let input = string_vec!["rm", dir];
        let result = rm(&input);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), format!("Is a directory: {}", dir));
        assert!(
            Path::new(dir).exists(),
            "Directory should not have been removed"
        );
    }

    #[test]
    fn rm_files_and_directories_fails() {
        let temp_store = TempStore::new(4);
        let file1 = &temp_store.store[0];
        let file2 = &temp_store.store[1];
        let dir1 = &temp_store.store[2];
        let dir2 = &temp_store.store[3];

        fs::write(file1.to_string(), "").expect("failed to create test file");
        fs::write(file2.to_string(), "").expect("failed to create test file");
        fs::create_dir(dir1.to_string()).expect("failed to create test directory");
        fs::create_dir(dir2.to_string()).expect("failed to create test directory");

        let input = string_vec!["rm", file1, dir1, file2, dir2];
        let result = rm(&input);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            format!("Is a directory: {}\nIs a directory: {}", dir1, dir2)
        );
        assert!(!Path::new(file1).exists(), "File should have been removed");
        assert!(!Path::new(file2).exists(), "File should have been removed");
        assert!(
            Path::new(dir1).exists(),
            "Directory should not have been removed"
        );
        assert!(
            Path::new(dir2).exists(),
            "Directory should not have been removed"
        );
    }

    #[test]
    fn rm_recursive() {
        let temp_store = TempStore::new(4);
        let file1 = &temp_store.store[0];
        let file2 = &temp_store.store[1];
        let dir1_str = &temp_store.store[2];
        let dir1 = Path::new(dir1_str);
        let dir2 = Path::new(&temp_store.store[3]);

        fs::create_dir(dir1).expect("failed to create test directory");
        fs::create_dir(dir1.join(dir2)).expect("failed to create test directory");

        fs::write(dir1.join(file1), "").expect("failed to create test file");
        fs::write(dir1.join(dir2).join(file2), "").expect("failed to create test file");

        let input = string_vec!["rm", "-r", dir1_str];
        let result = rm(&input);

        assert!(result.is_ok(), "`rm` failed: {:?}", result.err());
        assert!(
            !Path::new(dir1).exists(),
            "Directory should have been removed"
        );
    }
}
