use std::process;

pub fn exit(input: &Vec<String>) -> Result<String, String> {
    if input.len() > 1 {
        return Err("too many arguments".to_string());
    }
    process::exit(0);
}
