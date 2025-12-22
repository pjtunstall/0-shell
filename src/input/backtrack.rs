pub fn find_matches(data: &[String], partial: &str) -> Vec<String> {
    let mut matches = Vec::new();
    backtrack(data, partial, &mut matches);
    matches
}

fn reject(partial: &str, data: &[String]) -> bool {
    !data.iter().any(|item| item.starts_with(partial))
}

fn accept(partial: &str, data: &[String]) -> bool {
    data.iter().any(|item| item == partial)
}

fn get_candidates(partial: &str, data: &[String]) -> Vec<String> {
    let mut candidates = Vec::new();
    let partial_len = partial.len();

    for item in data {
        if item.starts_with(partial) && item.len() > partial_len {
            if let Some(next_char) = item[partial_len..].chars().next() {
                let candidate = format!("{}{}", partial, next_char);

                if !candidates.contains(&candidate) {
                    candidates.push(candidate);
                }
            }
        }
    }

    candidates
}

fn backtrack(data: &[String], partial: &str, matches: &mut Vec<String>) {
    if reject(partial, data) {
        return;
    }

    if accept(partial, data) {
        matches.push(String::from(partial));
    }

    let candidates = get_candidates(partial, data);

    for candidate in candidates {
        backtrack(data, &candidate, matches);
    }
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;

    use super::find_matches;
    use crate::string_vec;

    lazy_static! {
        static ref COMMANDS: Vec<String> = string_vec![
            "cat", "cd", "cp", "echo", "exit", "ls", "mkdir", "mv", "pwd", "rm", "touch"
        ];
    }

    #[test]
    fn test_find_matches() {
        let mut expected;

        expected = Vec::new();
        assert_eq!(
            find_matches(&COMMANDS, "x"),
            expected,
            "`find_matches` should return an empty vector when there are no matches"
        );

        expected = vec![String::from("cat"), String::from("cd"), String::from("cp")];
        assert_eq!(
            find_matches(&COMMANDS, "c"),
            expected,
            "`find_matches(\"c\")` should find all three commands beginning with 'c'"
        );

        expected = vec![String::from("mkdir")];
        assert_eq!(
            find_matches(&COMMANDS, "mk"),
            expected,
            "`find_matches(\"mk\", true)` should return a vector containing just \"mkdir\""
        );
    }
}
