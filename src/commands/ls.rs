pub mod format;
mod system;

use std::{fs::File, io::Write, path::Path};

use super::redirect;

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

    let flags = LsFlags::parse(&sources)?;
    let first_pathname_index;
    match flags.first_pathname_index {
        Some(i) => first_pathname_index = i,
        None => match list_current_directory(&flags) {
            Ok(res) => {
                if targets.is_empty() {
                    return Ok(res);
                } else {
                    redirect(targets, res);
                    return Ok(String::new());
                }
            }
            Err(e) => return Err(e),
        },
    }

    let paths = &sources[first_pathname_index..];
    let PathClassification {
        directories,
        mut files,
        mut non_existent,
    } = classify_paths(paths);

    non_existent.sort();
    files.sort();

    let mut running_results = String::new();
    if targets.is_empty() {
        running_results.push_str(&non_existent.join(""));
    }
    process_files(&files, &flags, &mut running_results)?;
    let results = process_directories(input, directories, running_results, flags.as_u8(), files);

    return if targets.is_empty() || results.is_err() {
        results
    } else {
        println!("{}", non_existent.join("").trim_end());
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

fn list_current_directory(flags: &LsFlags) -> Result<String, String> {
    let path = Path::new(".");
    if flags.long_format {
        format::get_long_list(flags.as_u8(), path)
    } else {
        format::get_short_list(flags.as_u8(), path)
    }
}

fn process_files(
    files: &[String],
    flags: &LsFlags,
    results: &mut String,
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
        results.push_str(&format::short_format_list(files.to_vec())?);
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
                "\x1b[31m{}: No such file or directory found\x1b[0m\x1b[1m\n",
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
            format::get_short_list(flags, path)?
        };

        results.push_str(&dir_listing);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::ls;

    #[test]
    fn test_ls() {
        let basic = vec!["ls".to_string()];
        let a = vec!["ls".to_string(), "-a".to_string()];
        let l = vec!["ls".to_string(), "-l".to_string()];
        let f = vec!["ls".to_string(), "-F".to_string()];
        let alf = vec!["ls".to_string(), "-alF".to_string()];
        let l_a_f = vec![
            "ls".to_string(),
            "-l".to_string(),
            "-a".to_string(),
            "-F".to_string(),
        ];

        let inputs = [basic, a, l, f, alf, l_a_f];

        for input in inputs {
            assert!(ls(&input).is_ok(), "`ls` should be ok for {:?}", input);
        }
    }
}
