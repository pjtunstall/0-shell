use std::env;

use home;

pub fn cd(input: &Vec<String>) -> Result<String, String> {
    debug_assert!(!input.is_empty(), "Input for `cd` should not be empty");
    debug_assert!(
        input[0] == "cd",
        "Input for `{}` should not be passed to `cd`",
        input[0]
    );

    if input.len() > 2 {
        return Err("too many arguments".to_string());
    }

    let path: &String = match input.get(1) {
        Some(path) => path,
        None => {
            if let Some(home_path) = home::home_dir() {
                env::set_current_dir(&home_path)
                    .map_err(|e| format!("failed to change directory: {}", e))?;
                return Ok(String::new());
            } else {
                return Err("could not determine home directory".to_string());
            }
        }
    };

    match env::set_current_dir(path) {
        Ok(_) => Ok(String::new()),
        Err(e) => Err(format!("{}: {}", path, e)),
    }
}
