use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Tell Cargo to rerun this script if any source files change.
    println!("cargo:rerun-if-changed=src");

    // Get the project root directory.
    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").expect("failed to get project root directory");

    // Define paths for the generated binary files.
    let debug_path = Path::new(&manifest_dir).join("target/debug/zero_shell");
    let release_path = Path::new(&manifest_dir).join("target/release/zero_shell");

    // Define the new names for the binaries.
    let new_debug_path = Path::new(&manifest_dir).join("target/debug/0-shell");
    let new_release_path = Path::new(&manifest_dir).join("target/release/0-shell");

    // Renaming the binary files only after the build completes.
    // Ensure that the build is done first (this script runs after Cargo build).

    // Rename debug binary if it exists.
    if debug_path.exists() {
        fs::rename(&debug_path, &new_debug_path)
            .expect("failed to rename debug binary if it exists");
        println!("Renamed debug binary to {}", new_debug_path.display());
    }

    // Rename release binary if it exists.
    if release_path.exists() {
        fs::rename(&release_path, &new_release_path)
            .expect("failed to rename release binary if it exists");
        println!("Renamed release binary to {}", new_release_path.display());
    }
}
