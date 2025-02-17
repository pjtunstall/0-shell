use std::fs;

use uuid::Uuid;

pub struct TempStore {
    pub source: String,
    pub target: String,
}

impl TempStore {
    pub fn new() -> Self {
        let source = Uuid::new_v4().to_string();
        let target = Uuid::new_v4().to_string();
        TempStore { source, target }
    }
}

impl Drop for TempStore {
    fn drop(&mut self) {
        if let Err(_) = fs::remove_file(&self.source) {
            // If it's not a file, attempt to remove it as a directory
            fs::remove_dir_all(&self.source).ok();
        }

        if let Err(_) = fs::remove_file(&self.target) {
            fs::remove_dir_all(&self.target).ok();
        }
    }
}
