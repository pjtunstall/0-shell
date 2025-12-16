use std::{fs, path::Path};

pub const USAGE: &str = "USAGE:\tmv SOURCE_FILE TARGET_DIRECTORY\n\tmv SOURCE_FILE TARGET_DIRECTORY/NEW_NAME\n\tmv SOURCE_FILE NEW_NAME";

pub fn mv(input: &[String]) -> Result<String, String> {
    validate_input(input)?;

    let target = &input[2];
    let source = &input[1];

    let source_path = Path::new(source);
    let target_path = Path::new(target);

    if target_path.is_dir() {
        let dest_file = target_path.join(
            source_path
                .file_name()
                .ok_or_else(|| "Failed to join source name to target".to_string())? // Convert `None` to `Err(String)`.
                .to_owned(), // Convert `&OsStr` to `OsString` (needed for join).
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

fn validate_input(input: &[String]) -> Result<(), String> {
    if input.len() < 3 {
        return Err(format!("Not enough arguments\n{}", USAGE).to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{MAIN_SEPARATOR, Path},
    };

    use uuid::Uuid;

    use super::{USAGE, mv};
    use crate::{string_vec, test_helpers::TempStore};

    #[test]
    fn mv_to_dir() {
        let temp_store = TempStore::new(2);

        let source = &temp_store.store[0];
        let source_contents = "hello";
        fs::write(&source, source_contents).expect("failed to create test source file");

        let target = &temp_store.store[1];
        fs::create_dir(target).expect("failed to create target directory");

        let input = string_vec!["mv", source, target];
        let result = mv(&input);

        assert!(result.is_ok(), "`mv` failed: {:?}", result.err());
        let path_to_moved = &format!("{}{}{}", target, MAIN_SEPARATOR, source);
        assert!(
            Path::new(path_to_moved).exists(),
            "File should be in target directory"
        );
        let moved_contents = fs::read_to_string(path_to_moved).expect("failed to read moved file");
        assert_eq!(
            moved_contents, source_contents,
            "File contents do not match"
        );
    }

    #[test]
    fn mv_as_rename() {
        let temp_store = TempStore::new(2);

        let source = &temp_store.store[0];
        let source_contents = "hello";
        fs::write(&source, source_contents).expect("failed to create test source file");

        let target = &temp_store.store[1];
        fs::write(&target, "world").expect("failed to create test source file");

        let input = string_vec!["mv", source, target];
        let result = mv(&input);

        assert!(result.is_ok(), "`mv` failed: {:?}", result.err());
        assert!(Path::new(target).exists(), "Renamed file should exist");
        assert!(
            !Path::new(source).exists(),
            "File should not exist still under old name"
        );

        let new_target_contents = fs::read_to_string(target).expect("failed to read moved file");
        assert_eq!(
            new_target_contents, source_contents,
            "File contents should match"
        );
    }

    #[test]
    fn mv_as_rename_when_new_name_already_exists() {
        let temp_store = TempStore::new(2);
        let source = &temp_store.store[0];

        let source_contents = "hello";
        fs::write(source, source_contents).expect("failed to create test source file");

        let target = &temp_store.store[1];
        fs::write(&target, "world").expect("failed to create test source file");

        let input = string_vec!["mv", source, target];
        let result = mv(&input);

        assert!(result.is_ok(), "`mv` should not fail: {:?}", result.err());
        assert!(Path::new(target).exists(), "Renamed file should exist");
        assert!(
            !Path::new(source).exists(),
            "File should not exist still under old name"
        );

        let new_target_contents =
            fs::read_to_string(target).expect("Should be able to read new file");
        assert_eq!(
            new_target_contents, source_contents,
            "File contents should match"
        );
    }

    #[test]
    fn mv_to_directory_and_rename() {
        let temp_store = TempStore::new(2);

        let source = &temp_store.store[0];
        let source_contents = "hello";
        fs::write(&source, source_contents)
            .expect(format!("failed to create test source file {}", source).as_str());

        let target = Path::new(&temp_store.store[1]);
        fs::create_dir(target)
            .expect(format!("failed to create target directory {}", target.display()).as_str());

        let binding = Uuid::new_v4().to_string();
        let new_name = Path::new(&binding);
        let path = target.join(new_name);
        let path_str = format!("{}", path.display());

        let input = string_vec!["mv", source, path_str];
        let result = mv(&input);

        assert!(result.is_ok(), "`mv' failed: {:?}", result.err());
        assert!(path.exists(), "renamed file should exist");
        assert!(
            !Path::new(source).exists(),
            "file should no longer exist under old name"
        );

        let moved_contents = fs::read_to_string(path).expect("failed to read moved file");
        assert_eq!(
            moved_contents, source_contents,
            "file contents should match"
        );
    }

    #[test]
    fn mv_insufficient_arguments_fails() {
        let file = Uuid::new_v4().to_string();

        let input = string_vec!["mv", file];
        let result = mv(&input);

        assert!(!result.is_ok(), "result should not be ok");

        let expected = Err(format!("Not enough arguments\n{}", USAGE).to_string());
        assert_eq!(result, expected, "Result should show correct error message");
    }

    #[test]
    fn mv_with_cycle_fails() {
        let temp_store = TempStore::new(2);
        let mover = &temp_store.store[0];
        let subdirectory = &temp_store.store[1];
        let subdirectory = format!(
            "{}",
            Path::new(mover).join(Path::new(subdirectory)).display()
        );

        fs::create_dir(mover).expect("failed to create directory to be moved");
        fs::create_dir(&subdirectory).expect("failed to create subdirectory");

        let input = string_vec!["mv", mover, mover];
        let output = mv(&input);

        assert!(
            output.is_err(),
            "Moving a directory into one of itself should be an error"
        );

        let input = string_vec!["mv", mover, subdirectory];
        let output = mv(&input);

        assert!(
            !output.is_ok(),
            "Moving a directory into one of its own subdirectories should be an error"
        );
    }
}
