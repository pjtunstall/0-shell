use std::env;

use super::helpers;

pub fn pwd(input: &[String]) -> Result<String, String> {
    debug_assert!(!input.is_empty(), "Input for `pwd` should not be empty");
    debug_assert!(
        input[0] == "pwd",
        "Input for `{}` should not be passed to `pwd`",
        input[0]
    );

    if let Err(err) = helpers::check_num_args(input, 1) {
        return Err(err);
    }

    let cwd = match env::current_dir() {
        Ok(cwd) => format!("{}", cwd.display()),
        Err(err) => {
            return Err(format!("getcwd: {}", err)
                .split(" (os")
                .next()
                .unwrap_or(" ")
                .to_string());
        }
    };

    let ok = format!("{}\n", cwd);
    Ok(ok)
}

#[cfg(test)]
mod tests {
    use std::path::MAIN_SEPARATOR;

    use super::pwd;
    use crate::input::split::split;

    #[test]
    fn test_pwd_success() {
        let input = "pwd";
        let expected = "0-shell\n";
        let result = pwd(&split(input).unwrap()).unwrap();
        let last_segment = result.split(MAIN_SEPARATOR).last().unwrap();
        assert_eq!(last_segment, expected);
    }

    #[test]
    fn test_pwd_too_many_args() {
        let input = "pwd foo";
        let expected = Err("too many arguments".to_string());
        assert_eq!(pwd(&split(input).unwrap()), expected);
    }
}
