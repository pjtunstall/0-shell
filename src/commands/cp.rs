use std::fs;
use std::path::Path;

fn is_directory(path: &str) -> bool {
    fs::metadata(path)
        .map(|meta| meta.is_dir())
        .unwrap_or(false)
}

pub fn cp(input: &Vec<String>) -> Result<String, String> {
    if input.len() < 3 {
        return Err("not enough arguments".to_string());
    }

    let sources = &input[1..input.len() - 1];
    let destination = &input[input.len() - 1];

    let dest_path = Path::new(destination);

    if sources.len() > 1 {
        if !is_directory(destination) {
            return Err(
                "target must be an existing directory when copying multiple sources\nusage: source_file target_file\n\tsource_file ... target_directory".to_string(),
            );
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
    use uuid::Uuid;

    use super::*;

    #[test]
    fn test_cp() {
        let source = Uuid::new_v4().to_string();
        let target = Uuid::new_v4().to_string();
        let input = vec!["cp".into(), source.clone(), target.clone()];

        let content = "Hello, cruel world!";
        fs::write(&source, content).expect("Failed to create test source file");

        let result = cp(&input);
        assert!(result.is_ok(), "cp failed: {:?}", result.err());
        assert!(Path::new(&target).exists(), "File not created");
        let copied_content = fs::read_to_string(&target).expect("Failed to read target file");
        assert_eq!(copied_content, content, "File contents do not match");

        fs::remove_file(source).ok();
        fs::remove_file(target).ok();
    }
}
