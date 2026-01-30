use std::fs::{create_dir_all, read_to_string, remove_dir_all, remove_file};
use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

use anyhow::Result;

pub fn create_directory(directory_path: &Path) -> Result<()> {
    if !directory_path.exists() {
        create_dir_all(directory_path).map_err(|e| {
            anyhow::Error::new(e).context(format!(
                "unable to create {}",
                directory_path.to_str().unwrap_or_default()
            ))
        })?;
    }
    Ok(())
}

pub fn create_parent_directory(file_path: &Path) -> Result<()> {
    if let Some(path) = file_path.parent() {
        create_directory(path)?
    }
    Ok(())
}

pub fn delete_file(file_path: &Path) -> Result<()> {
    if file_path.exists() {
        remove_file(file_path).map_err(|e| {
            anyhow::Error::new(e).context(format!("unable to delete {}", file_path.display()))
        })?;
    }
    Ok(())
}

pub fn read_file(file_path: &Path) -> Result<Option<String>> {
    if file_path.exists() {
        let option = read_to_string(file_path).map(Some).map_err(|e| {
            anyhow::Error::new(e).context(format!("unable to read {}", file_path.display()))
        })?;
        Ok(option)
    } else {
        Ok(None)
    }
}

pub fn read_file_to_string(optional_file_path: &Option<String>) -> String {
    match optional_file_path.clone() {
        Some(file_path_as_string) => {
            let file_path = Path::new(&file_path_as_string);
            match file_path.exists() {
                true => read_file(file_path).unwrap_or_default().unwrap_or_default(),
                false => String::default(),
            }
        }
        None => String::default(),
    }
}

pub fn delete_file_or_directory(path: &Path) -> Result<()> {
    if path.exists() {
        if path.is_file() {
            remove_file(path).map_err(|e| {
                anyhow::Error::new(e).context(format!("unable to delete {}", path.display()))
            })?;
        } else {
            remove_dir_all(path).map_err(|e| {
                anyhow::Error::new(e).context(format!("unable to delete {}", path.display()))
            })?;
        }
    }
    Ok(())
}

/// Check if the `dot` binary is available on the system.
/// The result is cached after the first check to avoid repeated system calls.
pub fn is_dot_available() -> bool {
    static DOT_AVAILABLE: OnceLock<bool> = OnceLock::new();
    *DOT_AVAILABLE.get_or_init(|| {
        Command::new("dot")
            .arg("-V")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    })
}

/// Check if the GRAPHVIZ_DOT environment variable is set.
pub fn is_graphviz_dot_set() -> bool {
    std::env::var("GRAPHVIZ_DOT").is_ok()
}

/// Determine if we should automatically add the smetana layout argument.
/// Returns true if:
/// - BOTH dot is not available AND GRAPHVIZ_DOT is not set
/// - AND the user hasn't already specified a layout argument
pub fn should_add_smetana_layout(user_args: &[String]) -> bool {
    // Check if either dot is available OR GRAPHVIZ_DOT is set
    // If either one is present, we assume GraphViz is configured
    let dot_available = is_dot_available();
    let graphviz_dot_set = is_graphviz_dot_set();
    let graphviz_configured = dot_available || graphviz_dot_set;
    
    // Check if user already specified a layout argument
    let has_layout_arg = user_args.iter().any(|arg| arg.starts_with("-Playout="));
    
    // Add smetana only if GraphViz is not configured and user hasn't specified a layout
    !graphviz_configured && !has_layout_arg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_add_smetana_layout_with_existing_layout() {
        let args = vec!["-png".to_string(), "-Playout=elk".to_string()];
        // Should not add smetana if user already specified a layout
        assert!(!should_add_smetana_layout(&args));
    }

    #[test]
    fn test_is_graphviz_dot_set() {
        // Test when GRAPHVIZ_DOT is not set
        std::env::remove_var("GRAPHVIZ_DOT");
        assert!(!is_graphviz_dot_set());
        
        // Test when GRAPHVIZ_DOT is set
        std::env::set_var("GRAPHVIZ_DOT", "/usr/bin/dot");
        assert!(is_graphviz_dot_set());
        
        // Clean up
        std::env::remove_var("GRAPHVIZ_DOT");
    }

    #[test]
    fn test_should_add_smetana_layout_with_graphviz_dot_set() {
        // When GRAPHVIZ_DOT is set, should not add smetana even if dot is not available
        std::env::set_var("GRAPHVIZ_DOT", "/usr/bin/dot");
        let args = vec!["-png".to_string()];
        assert!(!should_add_smetana_layout(&args));
        
        // Clean up
        std::env::remove_var("GRAPHVIZ_DOT");
    }
}
