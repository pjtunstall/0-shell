pub fn find_matches(data: &[String], input: &str) -> Vec<String> {
    let mut matches = Vec::new();
    backtrack(data, input, &mut matches);
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
            let next_char = item.chars().nth(partial_len).unwrap();
            let candidate = format!("{}{}", partial, next_char);

            if !candidates.contains(&candidate) {
                candidates.push(candidate);
            }
        }
    }

    candidates
}

pub fn backtrack(data: &[String], partial: &str, matches: &mut Vec<String>) {
    if reject(partial, data) {
        return;
    }

    if accept(partial, data) {
        matches.push(partial.to_string());
    }

    let candidates = get_candidates(partial, data);

    for candidate in candidates {
        backtrack(data, &candidate, matches);
    }
}

// Tested via the test for the `tab` function in `main`
