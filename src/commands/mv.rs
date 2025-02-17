use std::{fs, path::Path};

const USAGE: &str = "USAGE: source_file target_directory\n\tsource_file target_directory/new_name\n\tsource_file new_name";

pub fn mv(input: &Vec<String>) -> Result<String, String> {
    if input.len() < 3 {
        return Err(format!("not enough arguments\n{}", USAGE).to_string());
    }

    let target = &input[2];
    let source = &input[1];

    let source_path = Path::new(source);
    let target_path = Path::new(target);

    if source_path.is_dir() {
        return Err(format!("{} is a directory (not moved)", source).to_string());
    }

    if target_path.is_dir() {
        let dest_file = target_path.join(
            source_path
                .file_name()
                .expect("failed to join source name to target"),
        );
        fs::rename(source_path, dest_file).map_err(|err| err.to_string())?
    } else {
        fs::rename(source_path, target_path).map_err(|err| err.to_string())?
    }

    Ok("".to_string())
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::test_helpers::TempStore;

    #[test]
    fn test_mv_to_dir() {
        let temp_store = TempStore::new();
        let source = &temp_store.source;
        let target = &temp_store.target;

        let content = "Hello, cruel world!";
        fs::write(&source, content).expect("Failed to create test source file");
        fs::create_dir(target).expect("Failed to create target directory");

        let input = vec!["mv".into(), source.clone(), target.clone()];
        let result = mv(&input);
        assert!(result.is_ok(), "`mv' failed: {:?}", result.err());
        let path_to_moved = &format!("{}/{}", target, source);
        assert!(
            Path::new(path_to_moved).exists(),
            "File not found at destination"
        );
        let moved_content = fs::read_to_string(path_to_moved).expect("Failed to read moved file");
        assert_eq!(moved_content, content, "File contents do not match");
    }

    #[test]
    fn test_mv_as_rename() {
        let temp_store = TempStore::new();
        let source = &temp_store.source;
        let target = &temp_store.target;

        let content = "Hello, cruel world!";
        fs::write(&source, content).expect("Failed to create test source file");

        let input = vec!["mv".into(), source.clone(), target.clone()];
        let result = mv(&input);
        assert!(result.is_ok(), "`mv' failed: {:?}", result.err());

        assert!(Path::new(target).exists(), "Renamed file should exist");
        assert!(
            !Path::new(source).exists(),
            "File should not exist still under old name"
        );

        let moved_content = fs::read_to_string(target).expect("Failed to read moved file");
        assert_eq!(moved_content, content, "File contents do not match");
    }

    #[test]
    fn test_mov_to_dir_and_rename() {
        let mut temp_store = TempStore::new();
        let source = &temp_store.source;
        let target_dir = Uuid::new_v4().to_string();
        let new_name = Uuid::new_v4().to_string();
        temp_store.target = format!("{}/{}", target_dir.clone(), new_name);
        let target = &temp_store.target;

        let content = "Hello, cruel world!";
        fs::write(&source, content)
            .expect(format!("Failed to create test source file {}", source).as_str());
        fs::create_dir(target_dir.clone())
            .expect(format!("Failed to create target directory {}", target_dir).as_str());

        let input = vec!["mv".into(), source.clone(), target.clone()];
        let result = mv(&input);
        assert!(result.is_ok(), "`mv' failed: {:?}", result.err());

        assert!(Path::new(target).exists(), "Renamed file should exist");
        assert!(
            !Path::new(source).exists(),
            "File should not exist still under old name"
        );

        let moved_content = fs::read_to_string(target).expect("Failed to read moved file");
        assert_eq!(moved_content, content, "File contents do not match");
    }
}
