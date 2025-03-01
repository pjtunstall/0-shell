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
