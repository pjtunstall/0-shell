use std::{fs, path::Path};

const USAGE: &str = "Usage: rm [-r] FILE...";

pub fn rm(input: &Vec<String>) -> Result<String, String> {
    debug_assert!(!input.is_empty(), "Input for `rm` should not be empty");
    debug_assert!(
        input[0] == "rm",
        "Input for `{}` should not be passed to `rm`",
        input[0]
    );

    if input.len() < 2 {
        return Err(format!("not enough arguments\n{}", USAGE).to_string());
    }

    if input[1] == "-r" {
        if input.len() < 3 {
            return Err(format!("not enough arguments\n{}", USAGE).to_string());
        }

        let mut errors: Vec<Result<(), String>> = Vec::new();
        let mut i: u32 = 0;
        for arg in input[2..].iter() {
            let path = Path::new(&arg);

            let cmd = if i == 0 { "" } else { "rm: " };
            i += 1;

            if path.is_dir() {
                _ = fs::remove_dir_all(path).map_err(|err| {
                    errors.push(Err(format!("{}{}: {}", cmd, arg, err)));
                });
            } else {
                _ = fs::remove_file(path).map_err(|err| {
                    errors.push(Err(format!("{}{}: {}", cmd, arg, err)));
                });
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

    let mut errors: Vec<Result<(), String>> = Vec::new();
    let mut i: u32 = 0;
    for arg in input[1..].iter() {
        let path = Path::new(&arg);

        if path.is_dir() {
            let cmd = if i == 0 { "" } else { "rm: " };
            i += 1;
            errors.push(Err(format!("{}{}: is a directory", cmd, arg)));
        } else if let Err(err) = fs::remove_file(path) {
            errors.push(Err(format!("{}: {}", arg, err)));
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

    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::TempStore;

    #[test]
    fn test_rm_removes_one_file() {
        let temp_store = TempStore::new();
        let file = &temp_store.target;
        fs::write(file.to_string(), "").expect("Failed to create test file");

        let input = vec!["rm".to_string(), file.to_string()];
        let result = rm(&input);

        assert!(result.is_ok());
        assert!(!Path::new(file).exists(), "File should have been removed");
    }

    #[test]
    fn test_rm_removes_multiple_files() {
        let temp_store = TempStore::new();
        let file1 = &temp_store.source;
        let file2 = &temp_store.target;
        fs::write(file1.to_string(), "").expect("Failed to create test file");
        fs::write(file2.to_string(), "").expect("Failed to create test file");

        let input = vec!["rm".to_string(), file1.to_string(), file2.to_string()];
        let result = rm(&input);

        assert!(result.is_ok());
        assert!(!Path::new(file1).exists(), "File should have been removed");
        assert!(!Path::new(file2).exists(), "File should have been removed");
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

    #[test]
    fn test_rm_when_arguments_are_a_mixture_of_files_and_directories() {
        let file_store = TempStore::new();
        let file1 = &file_store.source;
        let file2 = &file_store.target;

        let dir_store = TempStore::new();
        let dir2 = &dir_store.source;
        let dir1 = &dir_store.target;

        fs::write(file1.to_string(), "").expect("Failed to create test file");
        fs::write(file2.to_string(), "").expect("Failed to create test file");
        fs::create_dir(dir1.to_string()).expect("Failed to create test directory");
        fs::create_dir(dir2.to_string()).expect("Failed to create test directory");

        let input = vec![
            "rm".to_string(),
            file1.to_string(),
            dir1.to_string(),
            file2.to_string(),
            dir2.to_string(),
        ];
        let result = rm(&input);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            format!("{}: is a directory\nrm: {}: is a directory", dir1, dir2)
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
    fn test_rm_recursive() {
        let file_store = TempStore::new();
        let file1 = &file_store.source;
        let file2 = &file_store.target;

        let dir_store = TempStore::new();
        let dir2 = &dir_store.source;
        let dir1 = &dir_store.target;

        fs::create_dir(dir1.to_string()).expect("Failed to create test directory");
        fs::create_dir(format!("{}/{}", dir1, dir2).to_string())
            .expect("Failed to create test directory");

        fs::write(format!("{}/{}", dir1, file1).to_string(), "")
            .expect("Failed to create test file");
        fs::write(format!("{}/{}/{}", dir1, dir2, file2).to_string(), "")
            .expect("Failed to create test file");

        let input = vec!["rm".to_string(), "-r".to_string(), dir1.to_string()];
        let result = rm(&input);

        assert!(result.is_ok(), "`rm` failed: {:?}", result.err());
        assert!(
            !Path::new(dir1).exists(),
            "Directory should have been removed"
        );
    }
}
