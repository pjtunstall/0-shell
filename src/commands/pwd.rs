use std::env;

pub fn pwd(input: &[String]) -> Result<String, String> {
    validate_input(input)?;

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

fn validate_input(input: &[String]) -> Result<(), String> {
    debug_assert!(!input.is_empty(), "Input for `pwd` should not be empty");
    debug_assert!(
        input[0] == "pwd",
        "Input for `{}` should not be passed to `pwd`",
        input[0]
    );

    if input.len() > 1 {
        return Err("Too many arguments".to_string());
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::MAIN_SEPARATOR;

    use super::pwd;
    use crate::string_vec;

    #[test]
    fn pwd_ok() {
        let input = string_vec!["pwd"];
        let expected = "0-shell\n";
        let result = pwd(&input).expect("`pwd` should be ok");
        let last_segment = result.split(MAIN_SEPARATOR).last().unwrap();
        assert_eq!(last_segment, expected);
    }

    #[test]
    fn pwd_too_many_args_fails() {
        let input = string_vec!["pwd", "foo"];
        let expected = Err("Too many arguments".to_string());
        assert_eq!(
            pwd(&input),
            expected,
            "Should be the correct error when more arguments than none are passed to `pwd`"
        );
    }
}
