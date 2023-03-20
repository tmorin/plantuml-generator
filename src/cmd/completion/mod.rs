use std::io;

use clap::ArgMatches;
use clap_complete::{generate, Shell};

use crate::cli::build_cli;
use crate::result::Result;

pub fn execute_completion(arg_matches: &ArgMatches) -> Result<()> {
    let shell = arg_matches.get_one::<Shell>("SHELL").unwrap();
    generate(
        *shell,
        &mut build_cli(),
        "plantuml-generator",
        &mut io::stdout(),
    );
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_completion() {
        let arg_matches =
            build_cli().get_matches_from(["plantuml-generator", "-l=Debug", "completion", "bash"]);
        execute_completion(arg_matches.subcommand_matches("completion").unwrap()).unwrap();
    }
}
