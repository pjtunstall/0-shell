use std::env;

pub const USAGE: &str = "Usage:\tpwd";

pub fn pwd(input: &[String]) -> Result<String, String> {
    is_input_len_at_least_two(input)?;

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

fn is_input_len_at_least_two(input: &[String]) -> Result<(), String> {
    if input.len() > 1 {
        return Err(format!("Too many arguments\n{}", USAGE));
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::MAIN_SEPARATOR;

    use super::{USAGE, pwd};
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
        let expected = Err(format!("Too many arguments\n{}", USAGE));
        assert_eq!(
            pwd(&input),
            expected,
            "Should be the correct error when more arguments than none are passed to `pwd`"
        );
    }
}
