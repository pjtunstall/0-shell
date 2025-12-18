pub const USAGE: &str = "Usage:\tsleep <SECONDS>";

pub fn sleep(input: &[String]) -> Result<String, String> {
    if input.len() < 2 {
        return Err(format!("Not enough arguments\n{}", USAGE));
    }

    if input.len() > 2 {
        return Err(format!("Too many arguments\n{}", USAGE));
    }

    match input[1].parse() {
        Ok(seconds) => {
            std::thread::sleep(std::time::Duration::from_secs(seconds));
            Ok(String::new())
        }
        Err(_) => Err(String::from("Failed to parse duration")),
    }
}
