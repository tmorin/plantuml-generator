use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;

use anyhow::Result;
use tera::{Function, Tera, Value};

struct ReadFileContentFunction {}

impl Function for ReadFileContentFunction {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let path_as_value = match args.get("path") {
            None => return Err(tera::Error::from("the argument `path` is missing")),
            Some(p) => p,
        };
        let path_as_string = match path_as_value.as_str() {
            None => {
                return Err(tera::Error::from(
                    "unable to convert the `path` to a string",
                ));
            }
            Some(p) => p,
        };
        let path = Path::new(path_as_string);
        let content = match read_to_string(path).map_err(tera::Error::from) {
            Ok(c) => c,
            Err(e) => {
                log::error!("unable to read {}", path_as_string);
                return Err(e);
            }
        };
        Ok(Value::String(content))
    }

    fn is_safe(&self) -> bool {
        false
    }
}

pub fn create_tera(
    templates: Vec<(&str, &str)>,
    additional_directory: Option<String>,
) -> Result<Tera> {
    let mut primary = Tera::default();

    primary
        .add_raw_templates(templates)
        .map_err(|e| anyhow::Error::new(e).context("unable to create the primary Tera instance"))?;
    primary.register_function("read_file_content", ReadFileContentFunction {});

    let tera = match additional_directory {
        None => primary,
        Some(directory) => {
            let secondary = Tera::parse(&directory).map_err(|e| {
                anyhow::Error::new(e).context("unable to create the secondary Tera instance")
            })?;
            primary.extend(&secondary).map_err(|e| {
                anyhow::Error::new(e).context("unable to extend the primary tera instance")
            })?;
            primary
        }
    };

    Ok(tera)
}
