use std::fs::read_to_string;
use std::path::Path;

use anyhow::Result;
use tera::{Kwargs, State, Tera, TeraResult, Value};

fn read_file_content(kwargs: Kwargs, _state: &State) -> TeraResult<Value> {
    let path_as_string: String = kwargs.must_get("path")?;
    let path = Path::new(&path_as_string);
    let content = read_to_string(path).map_err(|e| {
        log::error!("unable to read {}", path_as_string);
        tera::Error::chain(format!("unable to read {}", path_as_string), e)
    })?;
    Ok(Value::from(content))
}

pub fn create_tera(
    templates: Vec<(&str, &str)>,
    additional_directory: Option<String>,
) -> Result<Tera> {
    let mut tera = Tera::default();

    tera.register_function("read_file_content", read_file_content);
    tera.add_raw_templates(templates)
        .map_err(|e| anyhow::Error::new(e).context("unable to create the Tera instance"))?;

    if let Some(directory) = additional_directory {
        tera.load_from_glob(&directory).map_err(|e| {
            anyhow::Error::new(e).context("unable to load additional templates into Tera")
        })?;
    }

    Ok(tera)
}
