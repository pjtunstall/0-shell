use std::env;

pub fn echo(input: &Vec<String>) -> Result<String, String> {
    if input.len() < 2 {
        return Ok("\n".to_string());
    }

    let mut output = input[1].clone();

    output = parse_special_characters(&output);
    parse_environment_variables(&mut output);

    if input.len() > 2 {
        for i in 2..input.len() {
            output.push_str(" ");
            output.push_str(&input[i]);
        }
    }

    output.push('\n');
    Ok(output)
}

fn parse_special_characters(output: &str) -> String {
    output
        .replace(r"\n", "\n")
        .replace(r"\t", "\t")
        .replace(r"\v", "\x0b")
        .replace(r"\a", "\x07")
        .replace(r"\b", "\x08")
        .replace(r"\r", "\r")
        .replace(r"\\", "\\")
}

fn parse_environment_variables(output: &mut String) {
    *output = output.replace(
        "$USER",
        &env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
    );
    *output = output.replace(
        "$HOSTNAME",
        &env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string()),
    );
    *output = output.replace(
        "$PID",
        &env::var("PID").unwrap_or_else(|_| "unknown".to_string()),
    );
    *output = output.replace(
        "$PATH",
        &env::var("PATH").unwrap_or_else(|_| "unknown".to_string()),
    );
    *output = output.replace(
        "$SHELL",
        &env::var("SHELL").unwrap_or_else(|_| "unknown".to_string()),
    );
    *output = output.replace(
        "$UMASK",
        &env::var("UMASK").unwrap_or_else(|_| "unknown".to_string()),
    );
    *output = output.replace(
        "$HOME",
        &env::var("HOME").unwrap_or_else(|_| "unknown".to_string()),
    );
    *output = output.replace(
        "$LANG",
        &env::var("LANG").unwrap_or_else(|_| "unknown".to_string()),
    );
    *output = output.replace(
        "$TERM",
        &env::var("TERM").unwrap_or_else(|_| "unknown".to_string()),
    );
}
