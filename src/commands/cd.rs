use std::env;

pub fn cd(input: &Vec<String>) -> Result<String, String> {
    if let Err(err) = crate::helpers::check_num_args(input, 2) {
        return Err(err);
    }

    let path = match input.get(1) {
        Some(path) => path,
        None => return Err("missing argument".to_string()),
    };

    match env::set_current_dir(path) {
        Ok(_) => Ok(String::new()),
        Err(e) => Err(format!("{}: {}", path, e)),
    }
}
