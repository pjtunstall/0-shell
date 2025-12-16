use super::*;

const USAGE: &str = "Usage:\tman COMMAND";

pub fn man(input: &[String]) -> Result<String, String> {
    if input.len() < 2 {
        return Err(format!("What manual page do you want?\n{}", USAGE));
    }

    let mut result = String::new();
    let mut wouldbe_next;

    for (i, cmd) in input[1..].iter().enumerate() {
        if i > 0 {
            result.push_str("\n");
        }

        let next = match cmd.as_str() {
            "cat" => cat::USAGE,
            "cd" => cd::USAGE,
            "cp" => cp::USAGE,
            "echo" => echo::USAGE,
            "exit" => exit::USAGE,
            "ls" => ls::USAGE,
            "man" => man::USAGE,
            "mkdir" => mkdir::USAGE,
            "mv" => mv::USAGE,
            "pwd" => pwd::USAGE,
            "rm" => rm::USAGE,
            "sleep" => sleep::USAGE,
            "touch" => touch::USAGE,
            _ => {
                wouldbe_next = format!("\x1b[31mNo manual entry for {}\x1b[0m\x1b[1m", cmd).clone();
                &wouldbe_next
            }
        };

        result.push_str(next);
    }

    result.push('\n');

    Ok(result)
}
