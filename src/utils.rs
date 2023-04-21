use std::fs::{create_dir_all, read_to_string, remove_dir_all, remove_file};
use std::path::Path;

use crate::error::Error;
use crate::result::Result;

pub fn create_directory(directory_path: &Path) -> Result<()> {
    if !directory_path.exists() {
        create_dir_all(directory_path).map_err(|e| {
            Error::Cause(
                format!(
                    "unable to create {}",
                    directory_path.to_str().unwrap_or_default()
                ),
                Box::from(e),
            )
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
            Error::Cause(
                format!("unable to delete {}", file_path.display()),
                Box::from(e),
            )
        })?;
    }
    Ok(())
}

pub fn read_file(file_path: &Path) -> Result<Option<String>> {
    if file_path.exists() {
        let option = read_to_string(file_path).map(Some).map_err(|e| {
            Error::Cause(
                format!("unable to read {}", file_path.display()),
                Box::from(e),
            )
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
