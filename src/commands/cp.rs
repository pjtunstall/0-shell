use std::{fs, path::Path};

use crate::helpers;

pub fn cp(input: &Vec<String>) -> Result<String, String> {
    let usage = "usage: source_file target_file\n\tsource_file ... target_directory";

    if input.len() < 3 {
        return Err(format!("not enough arguments\n{}", usage).to_string());
    }

    let sources = &input[1..input.len() - 1];
    let destination = &input[input.len() - 1];

    let dest_path = Path::new(destination);

    if sources.len() > 1 {
        if !helpers::is_directory(destination) {
            return Err(format!(
                "target must be an existing directory when copying multiple sources\n{}",
                usage
            )
            .to_string());
        }
    }

    for source in sources {
        let src_path = Path::new(source);

        if src_path.is_dir() {
            return Err(format!("{} is a directory (not copied)", source).to_string());
        }

        let dest_file = if dest_path.is_dir() {
            dest_path.join(src_path.file_name().unwrap())
        } else {
            dest_path.to_path_buf()
        };

        fs::copy(source, dest_file).map_err(|err| err.to_string())?;
    }

    Ok("".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::TempStore;

    #[test]
    fn test_cp() {
        let temp_store = TempStore::new();
        let source = &temp_store.source;
        let target = &temp_store.target;
        let input = vec!["cp".into(), source.clone(), target.clone()];

        let content = "Hello, cruel world!";
        fs::write(&source, content).expect("Failed to create test source file");

        let result = cp(&input);
        assert!(result.is_ok(), "`cp' failed: {:?}", result.err());
        assert!(Path::new(target).exists(), "File not created");
        let copied_content = fs::read_to_string(&target).expect("Failed to read target file");
        assert_eq!(copied_content, content, "File contents do not match");

        fs::remove_file(source).ok();
        fs::remove_file(target).ok();
    }
}
