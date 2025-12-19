use std::env;

use zero_shell::{commands, error, repl};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        if args[1] == "--internal-worker" {
            commands::run_command_as_worker(&args);
        } else {
            error::red_println("Usage: ./0_shell");
            return;
        }
    } else if args.len() > 1 {
        // TODO: If I implement scripting, replace this error with e.g. `run_script(&args[1]);`
        error::red_println("Usage: ./0_shell");
        return;
    } else {
        repl::repl();
    }
}
