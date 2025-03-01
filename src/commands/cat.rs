use std::{
    fs::File,
    io::{self, BufRead, Read, Write},
    path::Path,
};

fn separate_sources_and_targets(input: &[String]) -> (Vec<&String>, Vec<[&String; 2]>) {
    let mut sources = Vec::new();
    let mut targets = Vec::new();

    for (index, current) in input.iter().enumerate() {
        if index == 0 || current == ">" || current == ">>" {
            continue;
        }

        let previous = if index > 0 {
            input.get(index - 1)
        } else {
            None
        };

        if let Some(previous) = previous {
            if previous == ">" || previous == ">>" {
                targets.push([previous, current]);
            } else {
                sources.push(current);
            }
        } else {
            sources.push(current);
        }
    }

    (sources, targets)
}

pub fn cat(input: &[String]) -> Result<String, String> {
    debug_assert!(!input.is_empty(), "Input for `cat` should not be empty");
    debug_assert!(
        input[0] == "cat",
        "Input for `{}` should not be passed to `cat`",
        input[0]
    );

    let (sources, targets) = separate_sources_and_targets(input);

    if input.len() < 2 {
        return match get_input() {
            Ok(contents) => Ok(contents),
            Err(e) => Err(format!(
                "{}: {}",
                "cat",
                e.to_string()
                    .split(" (os ")
                    .next()
                    .unwrap_or(" ")
                    .to_string()
            )),
        };
    }

    let mut concatenated_contents = String::new();
    let mut errors = Vec::new();

    for &path_str in sources.iter() {
        let path = Path::new(path_str);
        if path.exists() {
            if path.is_file() {
                let mut file = match File::open(path) {
                    Ok(file) => file,
                    Err(e) => {
                        errors.push(format!(
                            "{}: {}: {}",
                            "cat",
                            path_str,
                            e.to_string()
                                .split(" (os ")
                                .next()
                                .unwrap_or(" ")
                                .to_string()
                        ));
                        continue;
                    }
                };
                let mut contents = String::new();
                if let Err(e) = file.read_to_string(&mut contents) {
                    errors.push(format!(
                        "{}: {}: {}",
                        "cat",
                        path_str,
                        e.to_string()
                            .split(" (os ")
                            .next()
                            .unwrap_or(" ")
                            .to_string()
                    ));
                } else {
                    concatenated_contents.push_str(&contents);
                }
            } else {
                errors.push(format!("cat: {}: Is a directory", path_str));
            }
        } else {
            errors.push(format!("cat: {}: No such file or directory", path_str));
        }
    }

    if targets.is_empty() {
        println!("{}", concatenated_contents);
    } else {
        for target in targets.iter() {
            let target_path = Path::new(target[1]);
            if target_path.is_dir() {
                errors.push(format!("cat: {}: Is a directory", target[1]));
                continue;
            }

            if !target_path.exists() || target[0] == ">" {
                let mut file = File::create(target_path).unwrap();
                file.write_all(concatenated_contents.as_bytes()).unwrap();
            } else {
                let mut file = File::options()
                    .append(true)
                    .create(true)
                    .open(target_path)
                    .unwrap();
                file.write_all(concatenated_contents.as_bytes()).unwrap();
            }
        }
    }

    if errors.is_empty() {
        Ok(concatenated_contents)
    } else {
        let joined_errors = errors.join("\n");
        if let Some(suffix) = joined_errors.strip_prefix("cat: ") {
            Err(suffix.to_string())
        } else {
            Err(joined_errors)
        }
    }
}

fn get_input() -> Result<String, String> {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut line = String::new();
    let mut contents = String::new();

    loop {
        line.clear(); // Clear the buffer for each line.
        match handle.read_line(&mut line) {
            Ok(0) => {
                // EOF (Ctrl+D) reached, exit the loop.
                break;
            }
            Ok(_) => {
                if line.ends_with('\n') {
                    line.pop();
                    if line.ends_with('\r') {
                        line.pop();
                    }
                    contents.push_str(line.as_str());
                }
                println!("{}", line);
            }
            Err(e) => {
                return Err(format!(
                    "{}: {}: {}",
                    "cat",
                    "-",
                    e.to_string()
                        .split(" (os ")
                        .next()
                        .unwrap_or(" ")
                        .to_string()
                ));
            }
        }
    }

    Ok(contents)
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write, path::Path};

    use crate::test_helpers::TempStore;

    use super::cat;

    #[test]
    fn test_cat_success_one_existing_file() {
        let file = &TempStore::new(1).store[0];
        fs::write(file, "Howdie, world!\n").unwrap();

        let result = cat(&vec!["cat".to_string(), file.to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Howdie, world!\n");
    }

    #[test]
    fn test_cat_success_rediect_from_one_file_to_one_new_file() {
        let temp_store = TempStore::new(3);
        let source = &temp_store.store[0];
        let target_write = &temp_store.store[1];
        let target_append = &temp_store.store[2];
        let expected = "Alright, world?\n";

        fs::write(source, expected).unwrap();

        // Redirect with `>`:
        let write_result = cat(&vec![
            "cat".to_string(),
            source.to_string(),
            ">".to_string(),
            target_write.to_string(),
        ]);
        assert!(write_result.is_ok(), "`cat` with `>` should be ok");
        assert!(
            Path::new(target_write).exists(),
            "Failed to create `>` target file"
        );

        let contents = fs::read_to_string(target_write).expect("Failed to read from target file");
        assert_eq!(
            contents, expected,
            "Contents of new `>` target file do should match those of source file"
        );

        // Redirect with `>>`:
        let append_result = cat(&vec![
            "cat".to_string(),
            source.to_string(),
            ">>".to_string(),
            target_append.to_string(),
        ]);
        assert!(append_result.is_ok(), "`cat` with `>>` should be ok");
        assert!(
            Path::new(target_append).exists(),
            "Failed to create `>>` target file"
        );

        let contents = fs::read_to_string(target_append).expect("Failed to read from target file");
        assert_eq!(
            contents, expected,
            "Contents of new `>>` target file should match those of source file"
        );
    }

    #[test]
    fn test_cat_success_append_to_one_existing_file() {
        let temp_store = TempStore::new(3);
        let source = &temp_store.store[0];
        let target = &temp_store.store[1];

        let expected = "Hello, world!\n";

        fs::write(source, "world!\n").expect("Failed to write to source file");
        fs::write(target, "Hello, ").expect("Failed to write to target file");

        let result = cat(&vec![
            "cat".to_string(),
            source.to_string(),
            ">>".to_string(),
            target.to_string(),
        ]);
        assert!(result.is_ok(), "`cat` with `>>` should be ok");

        let contents = fs::read_to_string(target).expect("Failed to read from target file");
        assert_eq!(
            contents, expected,
            "Target file should have source contents appended to it"
        );
    }

    #[test]
    fn test_cat_success_two_existing_files() {
        let temp_store = TempStore::new(2);
        let temp_file1_path = &temp_store.store[0];
        let temp_file2_path = &temp_store.store[1];

        let mut file1 = fs::File::create(temp_file1_path).unwrap();
        file1.write_all(b"Hello, ").unwrap();

        let mut file2 = fs::File::create(temp_file2_path).unwrap();
        file2.write_all(b"world!").unwrap();

        let input = vec![
            "cat".to_string(),
            temp_file1_path.to_string(),
            temp_file2_path.to_string(),
        ];
        let result = cat(&input).unwrap();

        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_cat_fail_one_nonexistent_file() {
        let input = vec!["cat".to_string(), "nonexistent.txt".to_string()];
        let result = cat(&input);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No such file or directory"));
    }

    #[test]
    fn test_cat_fail_one_directory() {
        let dir = &TempStore::new(1).store[0];
        fs::create_dir(dir).unwrap();
        let input = vec!["cat".to_string(), dir.to_string()];
        let result = cat(&input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Is a directory"));
    }
}
