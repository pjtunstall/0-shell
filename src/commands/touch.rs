use std::fs::File;
use std::path::Path;

use filetime;

pub fn touch(input: &Vec<String>) -> Result<String, String> {
    debug_assert!(!input.is_empty(), "Input for `touch` should not be empty");
    debug_assert!(
        input[0] == "touch",
        "Input for `{}` should not be passed to `touch`",
        input[0]
    );

    if let Err(err) = crate::helpers::check_num_args(input, 2) {
        return Err(err);
    }

    let path = Path::new(&input[1]);

    if path.exists() {
        filetime::set_file_times(path, filetime::FileTime::now(), filetime::FileTime::now())
            .map_err(|e| e.to_string())?;
    } else {
        File::create(path).map_err(|e| e.to_string())?;
    }

    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::touch;
    use crate::test_helpers::TempStore;

    #[test]
    fn test_touch() {
        let temp_store = TempStore::new(1);
        let source = &temp_store.store[0];
        let path = Path::new(source);

        let input = vec![String::from("touch"), source.to_string()];
        let result = touch(&input);
        assert!(result.is_ok(), "Result should be ok");
        assert!(path.exists(), "New file should exist");
    }
}
