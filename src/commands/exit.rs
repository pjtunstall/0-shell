use std::process;

pub const USAGE: &str = "Usage:\texit";

pub fn exit(input: &[String]) -> Result<String, String> {
    if input.len() > 1 {
        return Err(format!("Too many arguments\n{}", USAGE));
    }
    process::exit(0);
}
