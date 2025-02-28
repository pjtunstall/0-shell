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
