pub fn separate_sources_from_targets(input: &[String]) -> (Vec<&String>, Vec<[&String; 2]>) {
    let mut sources = Vec::new();
    let mut targets = Vec::new();

    for (index, current) in input.iter().enumerate() {
        if index == 0 || current == ">" || current == ">>" {
            continue;
        }

        let previous = if index > 0 {
            input.get(index - 1)
        } else {
            None
        };

        if let Some(previous) = previous {
            if previous == ">" || previous == ">>" {
                targets.push([previous, current]);
            } else {
                sources.push(current);
            }
        } else {
            sources.push(current);
        }
    }

    (sources, targets)
}

#[cfg(test)]
mod tests {
    use super::separate_sources_from_targets;
    use crate::string_vec;

    #[test]
    fn separate_sources_from_targets_standard_case() {
        let input = string_vec!["cmd", "a", "b", ">", "c", ">>", "d", ">>", "e", "f"];
        let (sources, targets) = separate_sources_from_targets(&input);

        assert_eq!(sources, vec!["a", "b", "f"]);
        assert_eq!(targets, vec![[">", "c"], [">>", "d"], [">>", "e"]]);
    }
}
