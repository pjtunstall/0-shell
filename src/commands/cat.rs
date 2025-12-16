use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

use termion::{event::Key, input::TermRead, raw::IntoRawMode};

use crate::redirect;

pub const USAGE: &str = "Usage:\tcat [FILE]...";

pub fn cat(input: &[String]) -> Result<String, String> {
    let (sources, targets) = redirect::separate_sources_from_targets(input);

    // Handle input from stdin.
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
        print!("{}", concatenated_contents);
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
            match File::create(target_path) {
                Ok(mut file) => {
                    if let Err(_) = file.write_all(concatenated_contents.as_bytes()) {
                        errors.push(format!("Failed to write to file: {}", target[1]));
                    }
                }
                Err(_) => {
                    errors.push(format!("Failed to create file: {}", target[1]));
                }
            }
        } else {
            match File::options().append(true).create(true).open(target_path) {
                Ok(mut file) => {
                    if let Err(_) = file.write_all(concatenated_contents.as_bytes()) {
                        errors.push(format!("Failed to append to file: {}", target[1]));
                    }
                }
                Err(_) => {
                    errors.push(format!("Failed to open file: {}", target[1]));
                }
            }
        }
    }
}

fn get_input() -> Result<String, String> {
    let stdin = io::stdin();
    let mut stdout = io::stdout()
        .lock()
        .into_raw_mode()
        .expect("failed to enter raw mode"); // Enable raw mode
    let mut contents = String::new();

    for key in stdin.keys() {
        match key {
            Ok(Key::Ctrl('d')) | Ok(Key::Ctrl('c')) => break, // Ctrl+C and Ctrl+D exit the loop
            Ok(Key::Char('\n')) => {
                contents.push('\n');
                write!(stdout, "\r\n").expect("failed to write to `stdout`"); // Move to a new line and reset cursor position
                stdout.flush().expect("failed to flush `stdout`");
            }
            Ok(Key::Char(c)) => {
                contents.push(c);
                write!(stdout, "{}", c).expect("failed to write to `stdout`"); // Print character immediately
                stdout.flush().expect("failed to flush `stdout`");
            }
            Ok(_) => {}
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

fn format(s: &str, e: io::Error) -> String {
    format!(
        "{}: {}: {}",
        "cat",
        s,
        e.to_string()
            .split(" (os ")
            .next()
            .unwrap_or(" ")
            .to_string()
    )
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
                        errors.push(format(path_str, e));
                        continue;
                    }
                };

                let mut contents = String::new();
                if let Err(e) = file.read_to_string(&mut contents) {
                    errors.push(format(path_str, e));
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
    fn cat_one_source_file() {
        let file = &TempStore::new(1).store[0];
        fs::write(file, "Howdie, world!\n").expect("failed to write to file");

        let input = string_vec!["cat", file];
        let result = cat(&input).expect("result should be ok");
        assert_eq!(result, "Howdie, world!\n");
    }

    #[test]
    fn cat_rediect_from_one_file_to_one_new_file() {
        let temp_store = TempStore::new(3);
        let source = &temp_store.store[0];
        let expected = "Now then, world?\n";

        fs::write(source, expected).expect("failed to create source file");

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
            "failed to create `{}` target file",
            op
        );

        let contents = fs::read_to_string(target).expect("failed to read from target file");
        assert_eq!(
            contents, expected,
            "contents of new `{}` target file do should match those of source file",
            op
        );
    }

    #[test]
    fn cat_append_to_one_existing_file() {
        let temp_store = TempStore::new(3);
        let source = &temp_store.store[0];
        let target = &temp_store.store[1];

        let expected = "Hello, world!\n";

        fs::write(source, "world!\n").expect("failed to write to source file");
        fs::write(target, "Hello, ").expect("failed to write to target file");

        let input = string_vec!["cat", source, ">>", target];
        let result = cat(&input);

        assert!(result.is_ok(), "`cat` with `>>` should be ok");

        let contents = fs::read_to_string(target).expect("failed to read from target file");
        assert_eq!(
            contents, expected,
            "target file should have source contents appended to it"
        );
    }

    #[test]
    fn cat_two_source_files() {
        let temp_store = TempStore::new(2);
        let source1_string = &temp_store.store[0];
        let source2_string = &temp_store.store[1];

        let mut source1 =
            fs::File::create(source1_string).expect("failed to create 1st source file");
        source1
            .write_all(b"Hello, ")
            .expect("failed to write to 1st source file");

        let mut source2 =
            fs::File::create(source2_string).expect("failed to create 2nd source file");
        source2
            .write_all(b"world!")
            .expect("failed to write to 2nd source file");

        let input = string_vec!["cat", source1_string, source2_string];
        let result = cat(&input).expect("`cat` should be ok");

        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn cat_two_sources_and_two_targets() {
        let temp_store = TempStore::new(4);
        let source1_string = &temp_store.store[0];
        let source2_string = &temp_store.store[1];
        let target1_string = &temp_store.store[2];
        let target2_string = &temp_store.store[3];

        let mut source1 =
            fs::File::create(source1_string).expect("failed to create 1st source file");
        source1
            .write_all(b"Hello, ")
            .expect("failed to write to 1st source file");

        let mut source2 =
            fs::File::create(source2_string).expect("failed to create 2nd source file");
        source2
            .write_all(b"world!")
            .expect("failed to write to 2nd source file");

        let mut target1 =
            fs::File::create(target1_string).expect("failed to create 1st target file");
        target1
            .write_all(b"Oy! ")
            .expect("failed to write to 1st target file");

        let mut _target2 =
            fs::File::create(target2_string).expect("failed to create 2nd target file");

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
            fs::read_to_string(target1_string).expect("failed to read from 1st target file");
        assert_eq!(
            contents1, "Oy! Hello, world!",
            "1st target should have combined contents of both sources"
        );

        let contents2 =
            fs::read_to_string(target2_string).expect("failed to read from 2nd target file");
        assert_eq!(contents2, "Hello, world!");
    }

    #[test]
    fn cat_one_nonexistent_source_file_fails() {
        let input = string_vec!["cat", "nonexistent.txt"];
        let result = cat(&input);

        assert!(result.is_err(), "result should be an error");
        assert!(result.unwrap_err().contains("No such file or directory"));
    }

    #[test]
    fn cat_one_source_directory_fails() {
        let dir = &TempStore::new(1).store[0];
        fs::create_dir(dir).expect("failed to create would-be source directory");

        let input = string_vec!["cat", dir];
        let result = cat(&input);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Is a directory"));
    }
}
