/// Common test utilities shared across integration tests
use std::path::PathBuf;

/// Helper to get the binary path
pub fn get_binary_path() -> PathBuf {
    let mut path = std::env::current_exe()
        .expect("Failed to get current executable path")
        .parent()
        .expect("Failed to get parent directory")
        .parent()
        .expect("Failed to get parent directory")
        .to_path_buf();

    // Handle both debug and release builds
    if path.ends_with("deps") {
        path.pop();
    }

    path.push(format!(
        "plantuml-generator{}",
        std::env::consts::EXE_SUFFIX
    ));
    path
}
