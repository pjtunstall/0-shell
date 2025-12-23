mod child;
mod parent;

use std::{env, ffi::CString, ptr};

use crate::commands::jobs::Job;

// Store both items in a struct to ensure `c_strings` can't be dropped first, leaving
// the pointers dangling.
struct PreparedCommand {
    ptrs: Vec<*const i8>,
    c_strings: Vec<CString>,
}

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
    let cmd = prepare_command(is_worker, args)?;
    let pid = unsafe { libc::fork() };

    match pid {
        -1 => Err(String::from("Fork failed")),
        0 => {
            unsafe { child::run_child(cmd.ptrs, cmd.c_strings) };
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

fn prepare_command(is_worker: bool, args: &[String]) -> Result<PreparedCommand, String> {
    let c_strings = get_c_strings(is_worker, args)?;
    let mut ptrs: Vec<*const i8> = c_strings.iter().map(|s| s.as_ptr()).collect();
    ptrs.push(ptr::null());

    Ok(PreparedCommand { ptrs, c_strings })
}

fn get_c_strings(is_worker: bool, args: &[String]) -> Result<Vec<CString>, String> {
    let exec_args = get_exec_args(is_worker, args);
    exec_args
        .into_iter()
        .map(|s| CString::new(s).map_err(|_| String::from("Argument contains interior NUL byte")))
        .collect()
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
