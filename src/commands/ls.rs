pub mod format;
mod system;

use std::{
    collections::VecDeque,
    fs::{self, File},
    io::Write,
    path::Path,
};

use crate::{
    ansi::{BOLD, ERROR_COLOR, RESET},
    redirect,
};

pub const USAGE: &str = "Usage:\tls [-FalrR] [FILE|DIRECTORY]...";
pub const OPTIONS_USAGE: &str = "\r\n-F      -- append file type indicators\r\n-a      -- list entries starting with .\r\n-l      -- long listing\r\n-r      -- reverse order\r\n-R      -- recursive";

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
    reverse: bool,     // -r
    recursive: bool,   // -R
    first_pathname_index: Option<usize>,
}

impl LsFlags {
    fn parse(args: &Vec<&String>) -> Result<Self, String> {
        let mut flags = LsFlags {
            show_hidden: false,
            long_format: false,
            classify: false,
            reverse: false,
            recursive: false,
            first_pathname_index: None,
        };

        for (i, &arg) in args.iter().enumerate() {
            if !arg.starts_with('-') {
                flags.first_pathname_index = Some(i);
                break;
            }

            if arg
                .chars()
                .skip(1)
                .any(|c| !['a', 'l', 'F', 'r', 'R'].contains(&c))
            {
                // `skip(1)` to skip the '-'.
                return Err(format!("Unrecognized option: `{}'\n{}", arg, USAGE));
            }

            flags.show_hidden |= arg.contains('a');
            flags.long_format |= arg.contains('l');
            flags.classify |= arg.contains('F');
            flags.reverse |= arg.contains('r');
            flags.recursive |= arg.contains('R');
        }

        Ok(flags)
    }
}

pub fn ls(input: &[String]) -> Result<String, String> {
    let (sources, targets) = redirect::separate_sources_from_targets(input);
    let is_redirect = !targets.is_empty();

    let flags = LsFlags::parse(&sources)?;

    // Default to the current directory if no path is specified so we can reuse
    // the same flow (including recursion) for all cases.
    let default_path_holder = if flags.first_pathname_index.is_none() {
        Some(String::from("."))
    } else {
        None
    };
    let paths: Vec<&String> = if let Some(i) = flags.first_pathname_index {
        sources[i..].to_vec()
    } else {
        vec![
            default_path_holder
                .as_ref()
                .expect("missing default path for recursive ls"),
        ]
    };

    let mut path_classification = classify_paths(&paths);
    sort_path_classification(&mut path_classification, &flags);

    print_non_existent_paths(&path_classification.non_existent);

    let mut running_results =
        process_initial_results(&path_classification.files, &flags, is_redirect)?;

    finalize_directory_listing(
        input,
        path_classification,
        &mut running_results,
        &flags,
        is_redirect,
        targets,
    )
}

fn sort_path_classification(classification: &mut PathClassification, flags: &LsFlags) {
    classification.directories.sort();
    classification.non_existent.sort();
    classification.files.sort();

    if flags.reverse {
        classification.directories.reverse();
        classification.non_existent.reverse();
        classification.files.reverse();
    }
}

fn print_non_existent_paths(non_existent: &[String]) {
    print!("{}", non_existent.join(""));
}

fn process_initial_results(
    files: &[String],
    flags: &LsFlags,
    is_redirect: bool,
) -> Result<String, String> {
    let mut running_results = String::new();
    process_files(files, flags, &mut running_results, is_redirect)?;

    let mut trimmed_results = String::from(running_results.trim_start());
    if is_redirect && !trimmed_results.is_empty() {
        trimmed_results.push('\n');
    }

    Ok(trimmed_results)
}

fn finalize_directory_listing(
    input: &[String],
    path_classification: PathClassification,
    running_results: &mut String,
    flags: &LsFlags,
    is_redirect: bool,
    targets: Vec<[&String; 2]>,
) -> Result<String, String> {
    let results = process_directories(
        input,
        path_classification.directories,
        running_results.to_string(),
        flags,
        is_redirect,
    );

    if targets.is_empty() || results.is_err() {
        results
    } else {
        // This can't fail as we checked for errors above.
        redirect(targets, results.unwrap());
        Ok(String::new())
    }
}

fn redirect(targets: Vec<[&String; 2]>, contents: String) {
    for &target in targets.iter() {
        let target_path = Path::new(target[1]);
        if target_path.is_dir() {
            println!(
                "{ERROR_COLOR}0-shell: Is a directory: {path}{RESET}{BOLD}",
                path = target[1]
            );
        }

        if !target_path.exists() || target[0] == ">" {
            match File::create(target_path) {
                Ok(mut file) => {
                    if let Err(_) = file.write_all(contents.as_bytes()) {
                        println!(
                            "{ERROR_COLOR}0-shell: Failed to write to file: {path}{RESET}{BOLD}",
                            path = target[1]
                        );
                    }
                }
                Err(_) => {
                    println!(
                        "{ERROR_COLOR}0-shell: Failed to create file: {path}{RESET}{BOLD}",
                        path = target[1]
                    );
                }
            }
        } else {
            match File::options().append(true).create(true).open(target_path) {
                Ok(mut file) => {
                    if let Err(_) = file.write_all(contents.as_bytes()) {
                        println!(
                            "{ERROR_COLOR}0-shell: Failed to append to file: {path}{RESET}{BOLD}",
                            path = target[1]
                        );
                    }
                }
                Err(_) => {
                    println!(
                        "{ERROR_COLOR}0-shell: Failed to open file: {path}{RESET}{BOLD}",
                        path = target[1]
                    );
                }
            }
        }
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
            results.push_str(&format::get_long_list(flags, file_path, !is_redirect)?);
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
                "{ERROR_COLOR}ls: {path}: No such file or directory{RESET}{BOLD}\n",
                path = path_str
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
    flags: &LsFlags,
    is_redirect: bool,
) -> Result<String, String> {
    let mut results = results;
    let mut queue: VecDeque<String> = dirs.into();
    let show_header = flags.recursive || input.len() > 2;

    while let Some(dir) = queue.pop_front() {
        let path = Path::new(&dir);

        if !results.is_empty() {
            results.push('\n');
        }

        // Add directory header if there are multiple directories or non-dir files.
        if show_header {
            results.push_str(&format!("{}:\n", dir));
        }

        let dir_listing = if flags.long_format {
            format::get_long_list(flags, path, !is_redirect)?
        } else {
            format::get_short_list(flags, path, is_redirect)?
        };

        results.push_str(&dir_listing);

        if flags.recursive {
            let mut child_dirs: Vec<String> = fs::read_dir(path)
                .map(|entries| {
                    entries
                        .filter_map(Result::ok)
                        .filter_map(|entry| {
                            let child_path = entry.path();
                            let name = child_path.file_name()?.to_string_lossy().to_string();

                            // Avoid recursing into . or .. and honor hidden filtering.
                            if name == "." || name == ".." {
                                return None;
                            }
                            if !flags.show_hidden && system::is_hidden(&child_path) {
                                return None;
                            }
                            if child_path.is_dir() {
                                Some(child_path.display().to_string())
                            } else {
                                None
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();

            child_dirs.sort();
            if flags.reverse {
                child_dirs.reverse();
            }

            queue.extend(child_dirs);
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use std::{env, fs, path::Path};

    use super::ls;
    use crate::test_helpers::TempStore;

    #[test]
    fn test_basic_and_flags_ok() {
        let basic = vec![String::from("ls")];
        let a = vec![String::from("ls"), String::from("-a")];
        let l = vec![String::from("ls"), String::from("-l")];
        let f = vec![String::from("ls"), String::from("-F")];
        let alf = vec![String::from("ls"), String::from("-alF")];
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
    fn ls_redirect() {
        let temp_store = TempStore::new(1);
        let root_str = &temp_store.store[0];

        let root = Path::new(root_str);
        let file_1 = &root.join(Path::new("file_1"));
        let file_2 = &root.join(Path::new("file_2"));
        let folder_1 = &root.join(Path::new("folder_1"));
        let folder_2 = &root.join(Path::new("folder_2"));

        fs::create_dir_all(folder_1).expect("failed to create temp folder");
        fs::create_dir(folder_2).expect("failed to create temp folder");
        fs::write(file_1, "").expect("failed to create temp file");
        fs::write(file_2, "").expect("failed to create temp file");

        let file_a = folder_1.join(Path::new("file_a"));
        let file_b = folder_1.join(Path::new("file_b"));
        let folder_a = folder_1.join(Path::new("folder_a"));

        fs::write(file_a, "").expect("failed to create temp file");
        fs::write(file_b, "").expect("failed to create temp file");
        fs::create_dir(folder_a).expect("failed to create temp folder");

        let file_c = folder_2.join(Path::new("file_c"));
        let folder_c = folder_2.join(Path::new("folder_c"));
        let folder_d = folder_2.join(Path::new("folder_d"));

        fs::write(file_c, "").expect("failed to create temp file");
        fs::create_dir_all(folder_c).expect("failed to create temp folder");
        fs::create_dir_all(folder_d).expect("failed to create temp folder");

        let original_dir = env::current_dir().expect("failed to get current directory");
        env::set_current_dir(root).expect("failed to set current directory");

        let v = Path::new("v");
        fs::write(v, "prefix").expect("failed to write to temp file");

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

        let mut contents_of_u = fs::read_to_string(u).expect("failed to read target file `u`");
        contents_of_u = contents_of_u.replace("\r\n", "\n");
        let expected_u = "file_1\nfile_2\n\nfolder_1:\nfile_a\nfile_b\nfolder_a\n\nfolder_2:\nfile_c\nfolder_c\nfolder_d\n";

        let mut contents_of_v = fs::read_to_string(v).expect("failed to read target file `u`");
        contents_of_v = contents_of_v.replace("\r\n", "\n");
        let mut expected_v = String::from("prefix");
        expected_v.push_str(expected_u);

        env::set_current_dir(original_dir).expect("failed to set current directory");

        assert_eq!(
            contents_of_u, expected_u,
            "Contents of new target file `u` did not match expected"
        );
        assert_eq!(
            contents_of_v, expected_v,
            "Contents of existing target file `v` did not match expected"
        );
    }

    #[test]
    fn ls_recursive_basic() {
        let temp_store = TempStore::new(1);
        let root = Path::new(&temp_store.store[0]);

        fs::create_dir_all(root.join("dir_a").join("dir_aa")).expect("failed to create dirs");
        fs::create_dir_all(root.join("dir_b")).expect("failed to create dirs");
        fs::write(root.join("file_root"), "").expect("failed to create file");
        fs::write(root.join("dir_a").join("file_a1"), "").expect("failed to create file");

        let out_path = root.join("out");
        let root_abs = fs::canonicalize(root).expect("failed to canonicalize root");
        let input: Vec<String> = vec![
            "ls",
            "-R",
            root_abs.to_string_lossy().as_ref(),
            ">",
            out_path.to_string_lossy().as_ref(),
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let result = ls(&input);
        assert!(result.is_ok(), "`ls -R` should succeed on populated tree");

        let mut contents =
            fs::read_to_string(&out_path).expect("failed to read redirected ls output");
        contents = contents.replace("\r\n", "\n");

        let dir_a = root_abs.join("dir_a");
        let dir_b = root_abs.join("dir_b");
        let dir_aa = dir_a.join("dir_aa");

        let expected = format!(
            "{root}:\n\
dir_a\n\
dir_b\n\
file_root\n\
\n\
{dir_a}:\n\
dir_aa\n\
file_a1\n\
\n\
{dir_b}:\n\
\n\
{dir_aa}:\n",
            root = root_abs.display(),
            dir_a = dir_a.display(),
            dir_b = dir_b.display(),
            dir_aa = dir_aa.display(),
        );

        assert_eq!(
            contents, expected,
            "Recursive ls output did not match expected"
        );
    }

    #[test]
    fn ls_recursive_reverse_and_hidden() {
        let temp_store = TempStore::new(1);
        let root = Path::new(&temp_store.store[0]);

        fs::create_dir_all(root.join("dir_a").join("dir_aa")).expect("failed to create dirs");
        fs::create_dir_all(root.join("dir_b")).expect("failed to create dirs");
        fs::write(root.join("file_root"), "").expect("failed to create file");
        fs::write(root.join("dir_a").join("file_a1"), "").expect("failed to create file");
        fs::write(root.join("dir_b").join(".hidden_b"), "").expect("failed to create file");

        let out_path = root.join("out_hidden");
        let root_abs = fs::canonicalize(root).expect("failed to canonicalize root");
        let input: Vec<String> = vec![
            "ls",
            "-Rra",
            root_abs.to_string_lossy().as_ref(),
            ">",
            out_path.to_string_lossy().as_ref(),
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let result = ls(&input);
        assert!(result.is_ok(), "`ls -Rra` should succeed on populated tree");

        let mut contents =
            fs::read_to_string(&out_path).expect("failed to read redirected ls output");
        contents = contents.replace("\r\n", "\n");

        let dir_a = root_abs.join("dir_a");
        let dir_b = root_abs.join("dir_b");
        let dir_aa = dir_a.join("dir_aa");

        let expected = format!(
            "{root}:\n\
file_root\n\
{dir_b_name}\n\
{dir_a_name}\n\
..\n\
.\n\
\n\
{dir_b}:\n\
.hidden_b\n\
..\n\
.\n\
\n\
{dir_a}:\n\
file_a1\n\
{dir_aa_name}\n\
..\n\
.\n\
\n\
{dir_aa}:\n\
..\n\
.\n",
            root = root_abs.display(),
            dir_b = dir_b.display(),
            dir_a = dir_a.display(),
            dir_aa = dir_aa.display(),
            dir_b_name = dir_b.file_name().unwrap().to_string_lossy(),
            dir_a_name = dir_a.file_name().unwrap().to_string_lossy(),
            dir_aa_name = dir_aa.file_name().unwrap().to_string_lossy(),
        );

        assert_eq!(
            contents, expected,
            "Recursive ls -Rra output did not match expected"
        );
    }
}
