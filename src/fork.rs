mod child;
mod parent;

use std::{env, ffi::CString, ptr};

use crate::commands::jobs::Job;

pub fn spawn_job(
    args: &[String],
    jobs: &mut Vec<Job>,
    is_background_launch: bool,
    is_worker: bool,
    current: &mut usize,
    previous: &mut usize,
) -> Result<String, String> {
    // Build argv (command-line arguments) up front so that the child doesn't
    // have to allocate after the fork.
    let c_strings = get_c_strings(is_worker, args);
    let ptrs = get_pointers(&c_strings);

    let pid = unsafe { libc::fork() };

    match pid {
        -1 => Err(String::from("Fork failed")),
        0 => {
            child::run_child(ptrs, c_strings);
            unreachable!();
        }
        child_pid => parent::run_parent(
            args,
            jobs,
            is_background_launch,
            current,
            previous,
            child_pid,
        ),
    }
}

fn get_exec_args(is_worker: bool, args: &[String]) -> Vec<String> {
    if is_worker {
        let self_path = env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from("./0_shell"))
            .to_string_lossy()
            .into_owned();
        let mut v = vec![self_path, String::from("--internal-worker")];
        v.extend_from_slice(args);
        v
    } else {
        args.to_vec()
    }
}

fn get_pointers(c_strings: &[CString]) -> Vec<*const i8> {
    let mut ptrs: Vec<*const i8> = c_strings.iter().map(|s| s.as_ptr()).collect();
    ptrs.push(ptr::null());
    ptrs
}

fn get_c_strings(is_worker: bool, args: &[String]) -> Vec<CString> {
    get_exec_args(is_worker, args)
        .into_iter()
        .map(|s| {
            CString::new(s).unwrap_or_else(|_| {
                eprintln!("0-shell: argument contains interior NUL byte");
                std::process::exit(1);
            })
        })
        .collect()
}
