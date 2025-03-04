pub mod format;
mod system;

use std::{fs::File, io::Write, path::Path};

use crate::redirect;

const USAGE: &str = "Usage: ls [-F] [-a] [-l] [DIRECTORY]...";

pub const OPTIONS_USAGE: &str = "\r\n-F      -- append file type indicators\r\n-a      -- list entries starting with .\r\n-l      -- long listing";

struct PathClassification {
    directories: Vec<String>,
    files: Vec<String>,
    non_existent: Vec<String>,
}

#[derive(Debug)]
struct LsFlags {
    show_hidden: bool, // -a
    long_format: bool, // -l
    classify: bool,    // -F
    first_pathname_index: Option<usize>,
}

impl LsFlags {
    fn parse(args: &Vec<&String>) -> Result<Self, String> {
        let mut flags = LsFlags {
            show_hidden: false,
            long_format: false,
            classify: false,
            first_pathname_index: None,
        };

        for (i, &arg) in args.iter().enumerate() {
            if !arg.starts_with('-') {
                flags.first_pathname_index = Some(i);
                break;
            }

            if arg.chars().skip(1).any(|c| !['a', 'l', 'F'].contains(&c)) {
                // `skip(1)` to skip the '-'.
                return Err(format!("Unrecognized option `{}'\n{}", arg, USAGE));
            }

            flags.show_hidden |= arg.contains('a');
            flags.long_format |= arg.contains('l');
            flags.classify |= arg.contains('F');
        }

        Ok(flags)
    }

    fn as_u8(&self) -> u8 {
        let mut result = 0;
        if self.show_hidden {
            result |= 1;
        }
        if self.long_format {
            result |= 2;
        }
        if self.classify {
            result |= 4;
        }
        result
    }
}

pub fn ls(input: &[String]) -> Result<String, String> {
    debug_assert!(!input.is_empty(), "Input for `ls` should not be empty");
    debug_assert!(
        input[0] == "ls",
        "Input for `{}` should not be passed to `ls`",
        input[0]
    );

    let (sources, targets) = redirect::separate_sources_from_targets(input);
    let is_redirect = !targets.is_empty();

    let flags = LsFlags::parse(&sources)?;
    let first_pathname_index;
    match flags.first_pathname_index {
        Some(i) => first_pathname_index = i,
        None => match list_current_directory(&flags, is_redirect) {
            Ok(res) => {
                if is_redirect {
                    redirect(targets, res);
                    return Ok(String::new());
                } else {
                    return Ok(res);
                }
            }
            Err(e) => return Err(e),
        },
    }

    let paths = &sources[first_pathname_index..];
    let PathClassification {
        mut directories,
        mut files,
        mut non_existent,
    } = classify_paths(paths);

    directories.sort();
    non_existent.sort();
    files.sort();

    let mut running_results = String::new();
    print!("{}", non_existent.join(""));
    process_files(&files, &flags, &mut running_results, is_redirect)?;
    running_results = running_results.trim_start().to_string();
    if is_redirect && !running_results.is_empty() {
        running_results.push_str("\n");
    }
    let results = process_directories(
        input,
        directories,
        running_results,
        flags.as_u8(),
        files,
        is_redirect,
    );

    return if targets.is_empty() || results.is_err() {
        results
    } else {
        redirect(targets, results.unwrap());
        Ok(String::new())
    };
}

fn redirect(targets: Vec<[&String; 2]>, contents: String) {
    for &target in targets.iter() {
        let target_path = Path::new(target[1]);
        if target_path.is_dir() {
            println!(
                "\x1b[31m0-shell: Is a directory: {}\x1b[0m\x1b[1m",
                target[1]
            );
        }

        if !target_path.exists() || target[0] == ">" {
            let mut file = File::create(target_path).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        } else {
            let mut file = File::options()
                .append(true)
                .create(true)
                .open(target_path)
                .unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
    }
}

fn list_current_directory(flags: &LsFlags, is_redirect: bool) -> Result<String, String> {
    let path = Path::new(".");
    if flags.long_format {
        format::get_long_list(flags.as_u8(), path)
    } else {
        format::get_short_list(flags.as_u8(), path, is_redirect)
    }
}

fn process_files(
    files: &[String],
    flags: &LsFlags,
    results: &mut String,
    is_redirect: bool,
) -> Result<String, String> {
    if files.is_empty() {
        return Ok(results.to_string());
    }

    if flags.long_format {
        for file in files {
            let file_path = Path::new(file);
            results.push_str(&format::get_long_list(flags.as_u8(), file_path)?);
        }
    } else {
        if is_redirect {
            results.push_str(files.to_vec().join("\n").as_str());
        } else {
            results.push_str(&format::short_format_list(files.to_vec(), false)?);
        }
    }

    Ok(results.to_string())
}

fn classify_paths(paths: &[&String]) -> PathClassification {
    let mut directories = Vec::new();
    let mut files = Vec::new();
    let mut non_existent = Vec::new();

    for path_str in paths {
        let path = Path::new(path_str);
        if path.is_dir() {
            directories.push(path_str.to_string());
        } else if path.exists() {
            files.push(path_str.to_string());
        } else {
            non_existent.push(format!(
                "\x1b[31mls: {}: No such file or directory found\x1b[0m\x1b[1m\n",
                path_str
            ));
        }
    }

    PathClassification {
        directories,
        files,
        non_existent,
    }
}

fn process_directories(
    input: &[String],
    dirs: Vec<String>,
    results: String,
    flags: u8,
    files: Vec<String>,
    is_redirect: bool,
) -> Result<String, String> {
    let mut results = results;
    for (i, dir) in dirs.iter().enumerate() {
        let path = Path::new(dir);

        // Add spacing between sections.
        if i > 0 || !files.is_empty() {
            results.push_str("\n");
        }

        // Add directory header if there are multiple directories or non-dir files.
        if input.len() > 2 {
            results.push_str(&format!("{}:\n", dir));
        }

        let dir_listing = if flags & 2 != 0 {
            format::get_long_list(flags, path)?
        } else {
            format::get_short_list(flags, path, is_redirect)?
        };

        results.push_str(&dir_listing);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use std::{env, fs, path::Path};

    use super::ls;
    use crate::test_helpers::TempStore;

    #[test]
    fn test_ls() {
        let basic = vec!["ls".to_string()];
        let a = vec!["ls".to_string(), "-a".to_string()];
        let l = vec!["ls".to_string(), "-l".to_string()];
        let f = vec!["ls".to_string(), "-F".to_string()];
        let alf = vec!["ls".to_string(), "-alF".to_string()];
        let l_a_f = vec!["ls", "-l", "-a", "-F"]
            .into_iter()
            .map(String::from)
            .collect();

        let inputs = [basic, a, l, f, alf, l_a_f];

        for input in inputs {
            assert!(ls(&input).is_ok(), "`ls` should be ok for {:?}", input);
        }
    }

    #[test]
    fn test_redirect() {
        let temp_store = TempStore::new(1);
        let root_str = &temp_store.store[0];

        let root = Path::new(root_str);
        let file_1 = &root.join(Path::new("file_1"));
        let file_2 = &root.join(Path::new("file_2"));
        let folder_1 = &root.join(Path::new("folder_1"));
        let folder_2 = &root.join(Path::new("folder_2"));

        fs::create_dir_all(folder_1).expect("Failed to create temp folder");
        fs::create_dir(folder_2).expect("Failed to create temp folder");
        fs::write(file_1, "").expect("Failed to create temp file");
        fs::write(file_2, "").expect("Failed to create temp file");

        let file_a = folder_1.join(Path::new("file_a"));
        let file_b = folder_1.join(Path::new("file_b"));
        let folder_a = folder_1.join(Path::new("folder_a"));

        fs::write(file_a, "").expect("Failed to create temp file");
        fs::write(file_b, "").expect("Failed to create temp file");
        fs::create_dir(folder_a).expect("Failed to create temp folder");

        let file_c = folder_2.join(Path::new("file_c"));
        let folder_c = folder_2.join(Path::new("folder_c"));
        let folder_d = folder_2.join(Path::new("folder_d"));

        fs::write(file_c, "").expect("Failed to create temp file");
        fs::create_dir_all(folder_c).expect("Failed to create temp folder");
        fs::create_dir_all(folder_d).expect("Failed to create temp folder");

        let original_dir = env::current_dir().expect("Failed to get current directory");
        env::set_current_dir(root).expect("Failed to set current directory");

        let v = Path::new("v");
        fs::write(v, "prefix").expect("Failed to write to temp file");

        let input: Vec<String> = vec![
            "ls", "file_1", "file_2", "folder_1", ">", "u", "folder_2", ">>", "v",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let result = ls(&input);
        assert!(
            result.is_ok(),
            "Result of `ls file_1 file_2 folder_1 > u folder_2 >> v` should be ok"
        );

        let u = Path::new("u");
        assert!(u.exists(), "Target file `u` should have been created");
        assert!(v.exists(), "Target file `v` should have been created");

        let mut contents_of_u = fs::read_to_string(u).expect("Failed to read target file `u`");
        contents_of_u = contents_of_u.replace("\r\n", "\n");
        let expected_u = "file_1\nfile_2\n\nfolder_1:\nfile_a\nfile_b\nfolder_a\n\nfolder_2:\nfile_c\nfolder_c\nfolder_d\n";

        let mut contents_of_v = fs::read_to_string(v).expect("Failed to read target file `u`");
        contents_of_v = contents_of_v.replace("\r\n", "\n");
        let mut expected_v = String::from("prefix");
        expected_v.push_str(expected_u);

        env::set_current_dir(original_dir).expect("Failed to set current directory");

        assert_eq!(
            contents_of_u, expected_u,
            "Contents of new target file `u` did not match expected"
        );
        assert_eq!(
            contents_of_v, expected_v,
            "Contents of existing target file `v` did not match expected"
        );
    }
}
