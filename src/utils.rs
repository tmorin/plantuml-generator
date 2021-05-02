use std::fs::{create_dir_all, remove_dir_all, remove_file};
use std::path::Path;

use crate::error::Error;
use crate::result::Result;

pub fn create_parent_directory(file_path: &Path) -> Result<()> {
    if let Some(path) = file_path.parent() {
        if !path.exists() {
            create_dir_all(path).map_err(|e| {
                Error::Cause(
                    format!(
                        "unable to create {}",
                        file_path.to_str().unwrap_or_default()
                    ),
                    Box::from(e),
                )
            })?;
        }
    }
    Ok(())
}

pub fn delete_file(file_path: &Path) -> Result<()> {
    if file_path.exists() {
        remove_file(file_path).map_err(|e| {
            Error::Cause(
                format!("unable to delete {}", file_path.display()),
                Box::from(e),
            )
        })?;
    }
    Ok(())
}

pub fn delete_file_or_directory(path: &Path) -> Result<()> {
    if path.exists() {
        if path.is_file() {
            remove_file(path).map_err(|e| {
                Error::Cause(format!("unable to delete {}", path.display()), Box::from(e))
            })?;
        } else {
            remove_dir_all(path).map_err(|e| {
                Error::Cause(format!("unable to delete {}", path.display()), Box::from(e))
            })?;
        }
    }
    Ok(())
}
