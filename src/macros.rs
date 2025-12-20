// Transforms `string_vec[a, b, c, ...]` into a `Vec<String>` even if a, b, c, ... are different types of string, as long as each type has its own String::from function. Due to monomorphization, `into_iter().map(String::from).collect()` only works when the items are the same type.
#[cfg(test)]
#[macro_export]
macro_rules! string_vec {
    ($($s:expr),*) => {
        vec![$(String::from($s)),*]
    };
}

#[cfg(test)]
mod tests {
    use crate::string_vec;

    #[test]
    fn test_string_vec() {
        let input = string_vec!["a", String::from("b"), &String::from("c")];
        assert_eq!(
            input,
            vec![String::from("a"), String::from("b"), String::from("c")]
        );
    }
}
