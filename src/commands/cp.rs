use std::{fs, path::Path};

const USAGE: &str = "USAGE: source_file target_file\n\tsource_file ... target_directory";

pub fn cp(input: &Vec<String>) -> Result<String, String> {
    debug_assert!(!input.is_empty(), "Input for `cp` should not be empty");
    debug_assert!(
        input[0] == "cp",
        "Input for `{}` should not be passed to `cp`",
        input[0]
    );

    if input.len() < 3 {
        return Err(format!("not enough arguments\n{}", USAGE).to_string());
    }

    let sources = &input[1..input.len() - 1];
    let destination = &input[input.len() - 1];

    let dest_path = Path::new(destination);

    if sources.len() > 1 {
        if !dest_path.is_dir() {
            return Err(format!(
                "target must be an existing directory when copying multiple sources\n{}",
                USAGE
            )
            .to_string());
        }
    }

    for source in sources {
        let src_path = Path::new(source);

        if src_path.is_dir() {
            return Err(format!("`{}' is a directory (not copied)", source).to_string());
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
        assert!(result.is_ok(), "`cp` failed: {:?}", result.err());
        assert!(Path::new(target).exists(), "File not created");
        let copied_content = fs::read_to_string(&target).expect("Failed to read target file");
        assert_eq!(copied_content, content, "File contents do not match");
    }
}
