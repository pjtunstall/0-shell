use std::{
    process::Command,
    sync::atomic::{AtomicI32, Ordering},
    {collections::VecDeque, env},
};

use zero_shell::{
    commands::{cat, cd, cp, echo, exit, ls, man, mkdir, mv, pwd, rm, sleep, touch},
    input,
};

const SIGINT: i32 = 2;

// Stores the PID of the currently running foreground job.
// 0 means "no job running" (we're at the prompt).
static CURRENT_CHILD_PID: AtomicI32 = AtomicI32::new(0);

unsafe extern "C" {
    // Register the handler.
    fn signal(sig: i32, handler: extern "C" fn(i32)) -> usize;

    // Forward the signal to the child.
    fn kill(pid: i32, sig: i32) -> i32;
}

extern "C" fn handle_sigint(_signal: i32) {
    let pid = CURRENT_CHILD_PID.load(Ordering::Relaxed);
    if pid > 0 {
        unsafe {
            kill(pid, SIGINT);
        }
    }
}

struct TextStyle;

impl TextStyle {
    fn new() -> Self {
        print!("\x1b[1m"); // Be bold!

        TextStyle
    }
}

impl Drop for TextStyle {
    fn drop(&mut self) {
        // Reset formatting to normal when the item is dropped,
        print!("\x1b[0m"); // i.e. when the program ends.
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        if args[1] == "--internal-worker" {
            run_command_as_worker(&args);
        } else {
            red_println("Usage: ./0_shell");
            return;
        }
    } else if args.len() > 1 {
        // TODO: If I implement scripting, replace this error
        // with e.g. `run_script(&args[1]);`
        red_println("Usage: ./0_shell");
        return;
    } else {
        unsafe {
            // Register the handler.
            signal(SIGINT, handle_sigint);
        }

        let _style = TextStyle::new();
        let mut history = VecDeque::new();
        history.push_back(String::new());

        loop {
            let input_string = match input::get_input(&mut history) {
                Ok(ok_input) => ok_input,
                Err(err) => {
                    let text = format!("0-shell: failed to get input: {}", err);
                    red_println(&text);
                    continue;
                }
            };

            if input_string.is_empty() {
                continue;
            };
            history.push_back(input_string.clone());

            let input_after_splitting: Vec<String>;
            match input::split::split(&input_string) {
                Ok(res) => {
                    input_after_splitting = res;
                }
                Err(err) => {
                    red_println(&format!("0-shell: {}", &err));
                    continue;
                }
            }

            if input_after_splitting.is_empty() {
                red_println(&format!("0-shell: parse error near `\\n'"));
                continue;
            }

            run_command(&input_after_splitting);
        }
    }
}

fn run_command(args: &[String]) {
    let command = args[0].as_str();
    let result = match command {
        // State modifiers: necessarily built-in.
        "cd" => cd::cd(&args),
        "exit" => exit::exit(&args),

        // Lightweight utilities: built-in too.
        "echo" => echo::echo(&args),
        "pwd" => pwd::pwd(&args),

        // External utilities: delegate to worker process.
        "cat" | "cp" | "ls" | "mkdir" | "man" | "mv" | "rm" | "touch" | "sleep" => {
            launch_worker_process(&args)
        }
        _ => Err(format!("command not found: {}", command)),
    };

    match result {
        Ok(ok) => {
            if !ok.is_empty() {
                print!("{}", &ok);
            }
        }
        Err(err) => handle_error(command, err),
    }
}

fn launch_worker_process(args: &[String]) -> Result<String, String> {
    let self_path = env::current_exe().map_err(|e| format!("Unable to get own path: {}", e))?;

    let mut child = Command::new(self_path)
        .arg("--internal-worker")
        .args(args)
        .spawn()
        .expect("failed to spawn");

    // Add worker PID to store so that we can kill it.
    CURRENT_CHILD_PID.store(child.id() as i32, Ordering::Relaxed);

    // println!("Launched job with PID: {}", child.id());

    let status = child.wait();

    // Unregister child: stop targeting this PID.
    CURRENT_CHILD_PID.store(0, Ordering::Relaxed);

    let s = status.map_err(|e| format!("wait failed: {}", e))?;
    if s.success() || s.code().is_none() {
        // The code is `None` if the worker is terminated by SIGINT.
        Ok(String::new())
    } else {
        Err(format!("process terminated: {}", s))
    }
}

fn run_command_as_worker(args: &[String]) {
    if args.len() < 3 {
        red_println("0-shell: internal worker error: missing command argument");
        return;
    }

    let command = args[2].as_str();
    let clean_args = &args[2..];

    let result = match command {
        "sleep" => sleep::sleep(clean_args),
        "cat" => cat::cat(clean_args),
        "cp" => cp::cp(clean_args),
        "ls" => ls::ls(clean_args),
        "mkdir" => mkdir::mkdir(clean_args),
        "man" => man::man(clean_args),
        "mv" => mv::mv(clean_args),
        "rm" => rm::rm(clean_args),
        "touch" => touch::touch(clean_args),
        _ => Err(format!("command not found: {}", command)),
    };

    match result {
        Ok(ok) => {
            if !ok.is_empty() {
                print!("{}", &ok);
            }
        }
        Err(err) => handle_error(command, err),
    }
}

fn handle_error(command: &str, err: String) {
    if err.starts_with("0-shell: ") {
        red_println(&format!("{}", err));
    } else {
        red_println(&format!("{}: {}", command, err));
    }
}

fn red_println(text: &str) {
    println!("\x1b[31m{}\x1b[0m\x1b[1m", text);
}
