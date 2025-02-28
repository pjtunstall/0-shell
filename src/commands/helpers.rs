pub fn check_num_args(input: &[String], expected: usize) -> Result<String, String> {
    if input.len() > expected {
        return Err("Too many arguments".to_string());
    } else if input.len() < expected {
        return Err("Not enough arguments".to_string());
    }
    Ok(String::new())
}
