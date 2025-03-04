use std::{fs, path::Path};

const USAGE: &str = "USAGE: source_file target_file\n\tsource_file ... target_directory";

pub fn cp(input: &[String]) -> Result<String, String> {
    debug_assert!(!input.is_empty(), "Input for `cp` should not be empty");
    debug_assert!(
        input[0] == "cp",
        "Input for `{}` should not be passed to `cp`",
        input[0]
    );

    if input.len() < 3 {
        return Err(format!("Not enough arguments\n{}", USAGE).to_string());
    }

    let sources = &input[1..input.len() - 1];
    let destination = &input[input.len() - 1];

    let dest_path = Path::new(destination);

    if sources.len() > 1 {
        if !dest_path.is_dir() {
            return Err(format!(
                "Target must be an existing directory when copying multiple sources\n{}",
                USAGE
            )
            .to_string());
        }
    }

    for source in sources {
        let src_path = Path::new(source);

        if src_path.is_dir() {
            return Err(format!("`{}` is a directory (not copied)", source).to_string());
        }

        let dest_file = if dest_path.is_dir() {
            dest_path.join(src_path.file_name().unwrap())
        } else {
            dest_path.to_path_buf()
        };

        fs::copy(source, dest_file).map_err(|err| {
            err.to_string()
                .split(" (os ")
                .next()
                .unwrap_or(" ")
                .to_string()
        })?;
    }

    Ok("".to_string())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use super::cp;
    use crate::{string_vec, test_helpers::TempStore};

    #[test]
    fn cp_one_source_and_one_target() {
        let temp_store = TempStore::new(2);
        let source = &temp_store.store[0];
        let target = &temp_store.store[1];

        let content = "Hello, cruel world!";
        fs::write(&source, content).expect("Failed to create test source file");

        let input = string_vec!["cp", source, target];
        let result = cp(&input);

        assert!(result.is_ok(), "`cp` should be ok: {:?}", result.err());
        assert!(Path::new(target).exists(), "File not created");

        let copied_content = fs::read_to_string(&target).expect("Failed to read target file");
        assert_eq!(copied_content, content, "File contents do not match");
    }
}
