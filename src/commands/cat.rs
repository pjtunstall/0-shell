use std::{
    fs::File,
    io::{self, BufRead, Read, Write},
    path::Path,
};

use crate::redirect;

pub fn cat(input: &[String]) -> Result<String, String> {
    debug_assert!(!input.is_empty(), "Input for `cat` should not be empty");
    debug_assert!(
        input[0] == "cat",
        "Input for `{}` should not be passed to `cat`",
        input[0]
    );

    let (sources, targets) = redirect::separate_sources_from_targets(input);

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

    let (concatenated_contents, mut errors) = assemble_contents(sources);

    if targets.is_empty() {
        println!("{}", concatenated_contents);
    } else {
        redirect(targets, &concatenated_contents, &mut errors);
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

fn redirect(targets: Vec<[&String; 2]>, concatenated_contents: &str, errors: &mut Vec<String>) {
    for &target in targets.iter() {
        let target_path = Path::new(target[1]);
        if target_path.is_dir() {
            errors.push(format!("0-shell: Is a directory: {}", target[1]));
            break;
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

fn assemble_contents(sources: Vec<&String>) -> (String, Vec<String>) {
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

    (concatenated_contents, errors)
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write, path::Path};

    use super::cat;
    use crate::{string_vec, test_helpers::TempStore};

    #[test]
    fn test_cat_success_one_existing_file() {
        let file = &TempStore::new(1).store[0];
        fs::write(file, "Howdie, world!\n").unwrap();

        let input = string_vec!["cat", file];
        let result = cat(&input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Howdie, world!\n");
    }

    #[test]
    fn test_cat_success_rediect_from_one_file_to_one_new_file() {
        let temp_store = TempStore::new(3);
        let source = &temp_store.store[0];
        let expected = "Now then, world?\n";

        fs::write(source, expected).expect("Failed to create source file");

        redirect_helper(source, expected, ">");
        redirect_helper(source, expected, ">>");
    }

    fn redirect_helper(source: &str, expected: &str, op: &str) {
        let target = &TempStore::new(1).store[0];

        let input = string_vec!["cat", source, op, target];
        let result = cat(&input);

        assert!(result.is_ok(), "`cat` with `{}` should be ok", op);
        assert!(
            Path::new(target).exists(),
            "Failed to create `{}` target file",
            op
        );

        let contents = fs::read_to_string(target).expect("Failed to read from target file");
        assert_eq!(
            contents, expected,
            "Contents of new `{}` target file do should match those of source file",
            op
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

        let input = string_vec!["cat", source, ">>", target];
        let result = cat(&input);

        assert!(result.is_ok(), "`cat` with `>>` should be ok");

        let contents = fs::read_to_string(target).expect("Failed to read from target file");
        assert_eq!(
            contents, expected,
            "Target file should have source contents appended to it"
        );
    }

    #[test]
    fn test_cat_success_two_source_files() {
        let temp_store = TempStore::new(2);
        let source1_string = &temp_store.store[0];
        let source2_string = &temp_store.store[1];

        let mut source1 =
            fs::File::create(source1_string).expect("Failed to create first source file");
        source1
            .write_all(b"Hello, ")
            .expect("Failed to write to first source file");

        let mut source2 = fs::File::create(source2_string).expect("Failed to second source file");
        source2
            .write_all(b"world!")
            .expect("Failed to write to second source file");

        let input = string_vec!["cat", source1_string, source2_string];
        let result = cat(&input).expect("`cat` should be ok");

        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_cat_success_two_sources_and_two_targets() {
        let temp_store = TempStore::new(4);
        let source1_string = &temp_store.store[0];
        let source2_string = &temp_store.store[1];
        let target1_string = &temp_store.store[2];
        let target2_string = &temp_store.store[3];

        let mut source1 =
            fs::File::create(source1_string).expect("Failed to create first source file");
        source1
            .write_all(b"Hello, ")
            .expect("Failed to write to first source file");

        let mut source2 = fs::File::create(source2_string).expect("Failed to second source file");
        source2
            .write_all(b"world!")
            .expect("Failed to write to second source file");

        let mut target1 =
            fs::File::create(target1_string).expect("Failed to create first target file");
        target1
            .write_all(b"Oy! ")
            .expect("Failed to write to first target file");

        let mut _target2 =
            fs::File::create(target2_string).expect("Failed to create second target file");

        let input = string_vec![
            "cat",
            source1_string,
            source2_string,
            ">>",
            target1_string,
            ">>",
            target2_string
        ];
        let result = cat(&input).expect("`cat` should be ok");

        assert_eq!(result, "Hello, world!");

        let contents1 =
            fs::read_to_string(target1_string).expect("Failed to read from target file");
        assert_eq!(
            contents1, "Oy! Hello, world!",
            "First target should have combined contents of both sources"
        );

        let contents2 =
            fs::read_to_string(target2_string).expect("Failed to read from target file");
        assert_eq!(contents2, "Hello, world!");
    }

    #[test]
    fn test_cat_fail_one_nonexistent_source_file() {
        let input = string_vec!["cat", "nonexistent.txt"];
        let result = cat(&input);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No such file or directory"));
    }

    #[test]
    fn test_cat_fail_one_source_directory() {
        let dir = &TempStore::new(1).store[0];
        fs::create_dir(dir).expect("Failed to create would-be source directory");

        let input = string_vec!["cat", dir];
        let result = cat(&input);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Is a directory"));
    }
}
