use std::fs;

use uuid::Uuid;

pub struct TempStore {
    pub store: Vec<String>,
}

impl TempStore {
    pub fn new(n: usize) -> Self {
        let mut store = Vec::new();
        for _ in 0..n {
            store.push(Uuid::new_v4().to_string());
        }
        TempStore { store }
    }
}

impl Drop for TempStore {
    fn drop(&mut self) {
        for item in &self.store {
            if let Err(_) = fs::remove_file(&item) {
                // If it's not a file, attempt to remove it as a directory.
                fs::remove_dir_all(&item).ok();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_temp_store() {
            let path1_str;
            let path2_str;

            {
                let store = TempStore::new(2);
                assert_eq!(store.store.len(), 2);
                path1_str = store.store[0].clone();
                path2_str = store.store[1].clone();
                let path1 = Path::new(&path1_str);
                let path2 = Path::new(&path2_str);
                fs::write(path1, "Lorem ipsum, dude!").expect("Failed to write to file");
                fs::create_dir(path2).expect("Failed to create temp folder");
                assert!(path1.exists());
                assert!(path2.exists());
            }

            let path1 = Path::new(&path1_str);
            let path2 = Path::new(&path2_str);
            assert!(!path1.exists(), "File should have been removed");
            assert!(!path2.exists(), "Directory should have been removed");
        }
    }
}
