use crate::helpers;

pub fn pwd(input: &Vec<String>) -> Result<String, String> {
    if let Err(err) = helpers::check_num_args(input, 1) {
        return Err(err);
    }
    let cwd = match helpers::get_current_dir() {
        Ok(cwd) => cwd,
        Err(err) => return Err(format!("getcwd: {}", err)),
    };
    let ok = format!("{}\n", cwd);
    Ok(ok)
}

#[cfg(test)]
mod tests {
    use std::path::MAIN_SEPARATOR;

    use super::pwd;

    #[test]
    fn test_pwd_success() {
        let input = "pwd";
        let expected = "0-shell";
        let result = pwd(&crate::helpers::split(input)).unwrap();
        let last_segment = result.split(MAIN_SEPARATOR).last().unwrap();
        assert_eq!(last_segment, expected);
    }

    #[test]
    fn test_pwd_too_many_args() {
        let input = "pwd foo";
        let expected = Err("too many arguments".to_string());
        assert_eq!(pwd(&crate::helpers::split(input)), expected);
    }
}
