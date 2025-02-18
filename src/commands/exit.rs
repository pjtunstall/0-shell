use std::process;

pub fn exit(input: &Vec<String>) -> Result<String, String> {
    debug_assert!(!input.is_empty(), "Input for `exit` should not be empty");
    debug_assert!(
        input[0] == "exit",
        "Input for `{}` should not be passed to `exit`",
        input[0]
    );

    if input.len() > 1 {
        return Err("too many arguments".to_string());
    }
    process::exit(0);
}
