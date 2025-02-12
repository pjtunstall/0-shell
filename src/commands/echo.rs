pub fn echo(input: &Vec<String>) -> Result<String, String> {
    if let Err(err) = crate::helpers::check_num_args(input, 2) {
        return Err(err);
    }
    Ok(input[1].clone())
}
