use clap::ArgMatches;
use schemars::schema_for;

use crate::cmd::library::manifest::library::Library;
use crate::result::Result;

pub fn execute_library_schema(_arg_matches: &ArgMatches) -> Result<()> {
    log::info!("generate the JSON schema of the library");
    let schema = schema_for!(Library);
    log::info!("{}", serde_json::to_string_pretty(&schema).unwrap());
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::cli::build_cli;

    use super::*;

    #[test]
    fn test_generation() {
        let arg_matches =
            build_cli().get_matches_from(["plantuml-generator", "-l=Off", "library", "schema"]);
        execute_library_schema(
            arg_matches
                .subcommand_matches("library")
                .unwrap()
                .subcommand_matches("schema")
                .unwrap(),
        )
        .unwrap()
    }
}
