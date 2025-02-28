use std::{fs, path::Path};

const USAGE: &str = "USAGE: mv source_file target_directory\n\tmv source_file target_directory/new_name\n\tmv source_file new_name";

pub fn mv(input: &[String]) -> Result<String, String> {
    debug_assert!(!input.is_empty(), "Input for `mv` should not be empty");
    debug_assert!(
        input[0] == "mv",
        "Input for `{}` should not be passed to `mv`",
        input[0]
    );

    if input.len() < 3 {
        return Err(format!("Not enough arguments\n{}", USAGE).to_string());
    }

    let target = &input[2];
    let source = &input[1];

    let source_path = Path::new(source);
    let target_path = Path::new(target);

    if target_path.is_dir() {
        let dest_file = target_path.join(
            source_path
                .file_name()
                .ok_or_else(|| "Failed to join source name to target".to_string())? // Convert None to Err(String)
                .to_owned(), // Convert &OsStr to OsString (needed for join)
        );
        fs::rename(source_path, dest_file).map_err(|err| {
            err.to_string()
                .split(" (os ")
                .next()
                .unwrap_or(" ")
                .to_string()
        })?
    } else {
        fs::rename(source_path, target_path).map_err(|err| {
            err.to_string()
                .split(" (os ")
                .next()
                .unwrap_or(" ")
                .to_string()
        })?
    }

    Ok("".to_string())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{MAIN_SEPARATOR, Path},
    };

    use uuid::Uuid;

    use super::{USAGE, mv};
    use crate::test_helpers::TempStore;

    #[test]
    fn test_mv_to_dir() {
        let temp_store = TempStore::new(2);

        let source = &temp_store.store[0];
        let source_contents = "hello";
        fs::write(&source, source_contents).expect("Failed to create test source file");

        let target = &temp_store.store[1];
        fs::create_dir(target).expect("Failed to create target directory");

        let input = vec!["mv".to_string(), source.to_string(), target.to_string()];
        let result = mv(&input);

        assert!(result.is_ok(), "`mv` failed: {:?}", result.err());
        let path_to_moved = &format!("{}{}{}", target, MAIN_SEPARATOR, source);
        assert!(
            Path::new(path_to_moved).exists(),
            "File should be in target directory"
        );
        let moved_contents = fs::read_to_string(path_to_moved).expect("Failed to read moved file");
        assert_eq!(
            moved_contents, source_contents,
            "File contents do not match"
        );
    }

    #[test]
    fn test_mv_as_rename() {
        let temp_store = TempStore::new(2);

        let source = &temp_store.store[0];
        let source_contents = "hello";
        fs::write(&source, source_contents).expect("Failed to create test source file");

        let target = &temp_store.store[1];
        fs::write(&target, "world").expect("Failed to create test source file");

        let input = vec!["mv".to_string(), source.to_string(), target.to_string()];
        let result = mv(&input);

        assert!(result.is_ok(), "`mv` failed: {:?}", result.err());
        assert!(Path::new(target).exists(), "Renamed file should exist");
        assert!(
            !Path::new(source).exists(),
            "File should not exist still under old name"
        );

        let new_target_contents = fs::read_to_string(target).expect("Failed to read moved file");
        assert_eq!(
            new_target_contents, source_contents,
            "File contents should match"
        );
    }

    #[test]
    fn test_mv_as_rename_when_new_name_already_exists() {
        let temp_store = TempStore::new(2);
        let source = Path::new(&temp_store.store[0]);

        let source_contents = "hello";
        fs::write(&source, source_contents).expect("Failed to create test source file");

        let target = Path::new(&temp_store.store[1]);
        fs::write(&target, "world").expect("Failed to create test source file");

        let input = vec![
            "mv".to_string(),
            source
                .to_str()
                .expect("Unable to convert path to string")
                .to_string(),
            target
                .to_str()
                .expect("Unable to convert path to string")
                .to_string(),
        ];
        let result = mv(&input);

        assert!(result.is_ok(), "`mv' failed: {:?}", result.err());
        assert!(Path::new(target).exists(), "Renamed file should exist");
        assert!(
            !Path::new(source).exists(),
            "File should not exist still under old name"
        );

        let new_target_contents = fs::read_to_string(target).expect("Failed to read new file");
        assert_eq!(
            new_target_contents, source_contents,
            "File contents do not match"
        );
    }

    #[test]
    fn test_mv_to_directory_and_rename() {
        let temp_store = TempStore::new(2);

        let source = &temp_store.store[0];
        let source_contents = "hello";
        fs::write(&source, source_contents)
            .expect(format!("Failed to create test source file {}", source).as_str());

        let target = Path::new(&temp_store.store[1]);
        fs::create_dir(target)
            .expect(format!("Failed to create target directory {}", target.display()).as_str());

        let binding = Uuid::new_v4().to_string();
        let new_name = Path::new(&binding);
        let path = target.join(new_name);

        let input = vec![
            "mv".to_string(),
            source.to_string(),
            path.to_str()
                .expect("Unable to convert path to string")
                .to_string(),
        ];
        let result = mv(&input);

        assert!(result.is_ok(), "`mv' failed: {:?}", result.err());
        assert!(path.exists(), "Renamed file should exist");
        assert!(
            !Path::new(source).exists(),
            "File should no longer exist under old name"
        );

        let moved_contents = fs::read_to_string(path).expect("Failed to read moved file");
        assert_eq!(
            moved_contents, source_contents,
            "File contents should match"
        );
    }

    #[test]
    fn test_error_when_not_enough_arguments() {
        let file = Uuid::new_v4().to_string();
        let input = vec!["mv".to_string(), file];

        let result = mv(&input);
        assert!(!result.is_ok(), "Result should not be ok");
        let expected = Err(format!("Not enough arguments\n{}", USAGE).to_string());
        assert_eq!(result, expected, "Result should show correct error message");
    }

    #[test]
    fn test_error_on_cycle() {
        let temp_store = TempStore::new(2);
        let source = &temp_store.store[0];
        let target = &temp_store.store[1];

        fs::create_dir(source).expect("Failed to create source directory");
        fs::create_dir(target).expect("Failed to create target directory");

        let input = vec![String::from("mv"), target.clone(), target.clone()];
        let output = mv(&input);

        assert!(
            !output.is_ok(),
            "Moving a directory into one of its own subdirectories should be an error"
        );
    }
}
