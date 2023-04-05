use clap::builder::{PossibleValuesParser, ValueParser};
use clap::{
    crate_authors, crate_description, crate_version, value_parser, Arg, ArgAction, Command,
};
use clap_complete::Shell;

pub fn build_cli() -> Command {
    let arg_cache_directory: Arg = Arg::new("cache_directory")
        .short('C')
        .long("cache")
        .action(ArgAction::Set)
        .num_args(1)
        .env("PLANTUML_GENERATOR_OUTPUT_CACHE")
        .help("The cache directory.");

    let arg_plantuml_version: Arg = Arg::new("plantuml_version")
        .conflicts_with("plantuml_jar")
        .short('V')
        .long("plantuml-version")
        .action(ArgAction::Set)
        .num_args(1)
        .env("PLANTUML_GENERATOR_PLANTUML_VERSION")
        .help("The PlantUML version.");

    let arg_plantuml_jar: Arg = Arg::new("plantuml_jar")
        .conflicts_with("plantuml_version")
        .short('P')
        .long("plantuml")
        .action(ArgAction::Set)
        .num_args(1)
        .env("PLANTUML_GENERATOR_PLANTUML_JAR")
        .help("The PlantUML version.");

    let arg_java_binary: Arg = Arg::new("java_binary")
        .short('J')
        .long("java")
        .action(ArgAction::Set)
        .num_args(1)
        .env("PLANTUML_GENERATOR_JAVA_BINARY")
        .help("The java binary path or command line.");

    let arg_inkscape_binary: Arg = Arg::new("inkscape_binary")
        .short('I')
        .long("inkscape")
        .action(ArgAction::Set)
        .num_args(1)
        .env("PLANTUML_GENERATOR_INKSCAPE_BINARY")
        .help("The inkscape binary path or command line.");

    Command::new("plantuml-generator")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand_required(true)
        .arg_required_else_help(true)
        .arg(
            Arg::new("log_level")
                .short('l')
                .long("log-level")
                .action(ArgAction::Set)
                .num_args(1)
                .default_value("Info")
                .value_parser(PossibleValuesParser::new(["Off", "Trace", "Debug", "Info", "Warn", "Error"]))
                .help("Set the verbosity of the logs."),
        )
        .subcommand(
            Command::new("library")
                .about("Manage libraries")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("generate")
                        .about("Generate a library from a manifest.")
                        .arg(Arg::new("MANIFEST")
                            .index(1)
                            .required(true)
                            .action(ArgAction::Set)
                            .num_args(1)
                            .help("The manifest of the library.")
                        )
                        .arg(Arg::new("output_directory")
                            .short('O')
                            .long("output")
                            .env("PLANTUML_GENERATOR_OUTPUT_DIRECTORY")
                            .action(ArgAction::Set)
                            .num_args(1)
                            .help("The output directory.")
                        )
                        .arg(Arg::new("urns")
                            .help("Handle only artifacts included in the URN.")
                            .short('u')
                            .long("urn")
                            .action(ArgAction::Set)
                            .num_args(1)
                            .action(ArgAction::Append)
                            .value_parser(ValueParser::string())
                        )
                        .arg(Arg::new("do_clean_cache")
                            .long("clean-cache")
                            .action(ArgAction::SetTrue)
                            .help("Delete the cache directory before the generation-"))
                        .arg(Arg::new("urns_to_clean")
                            .help("Delete the given URN in the output directory before the generation.")
                            .long("clean-urn")
                            .action(ArgAction::Set)
                            .num_args(1)
                            .action(ArgAction::Append)
                            .value_parser(ValueParser::string())
                        )
                        .arg(Arg::new("cleanup_scopes")
                            .help("The scopes to cleanup before the generation.")
                            .long_help("By default, artifacts which are already generated won't be generated again. The cleanup-scope option helps to target artifacts which will be re-generated.")
                            .short('c')
                            .long("cleanup-scope")
                            .action(ArgAction::Set)
                            .num_args(1)
                            .action(ArgAction::Append)
                            .value_parser(PossibleValuesParser::new([
                                "All",
                                "Example",
                                "Item",
                                "ItemIcon",
                                "ItemSource",
                                "Snippet",
                                "SnippetSource",
                                "SnippetImage",
                                "Sprite",
                                "SpriteIcon",
                                "SpriteValue",
                            ]))
                        )
                        .arg(&arg_cache_directory)
                        .arg(&arg_plantuml_version)
                        .arg(&arg_plantuml_jar)
                        .arg(&arg_java_binary)
                        .arg(&arg_inkscape_binary),
                )
                .subcommand(
                Command::new("schema").about("Generate the JSON Schema of the library manifest.")
                ),

        )
        .subcommand(
            Command::new("diagram")
                .about("Manage diagrams")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("generate")
                        .about("Generate discovered .puml files which has been mutated since the last generation.")
                        .arg(Arg::new("source_directory")
                            .short('s')
                            .long("source")
                            .default_value(".")
                            .action(ArgAction::Set)
                            .num_args(1)
                            .env("PLANTUML_GENERATOR_SOURCE_DIRECTORY")
                            .help("The directory where the .puml will be discovered."))
                        .arg(Arg::new("do_force_generation")
                            .short('f')
                            .long("force")
                            .help("Force the rendering of discovered .puml file."))
                        .arg(&arg_cache_directory)
                        .arg(&arg_plantuml_version)
                        .arg(&arg_plantuml_jar)
                        .arg(&arg_java_binary)
                ),
        )
        .subcommand(Command::new("completion")
            .about("Generate resources for autocompletion")
            .arg_required_else_help(true)
            .arg(
                Arg::new("SHELL")
                    .help("set the shell")
                    .index(1)
                    .action(ArgAction::Set)
                    .num_args(1)
                    .required(true)
                    .value_parser(value_parser!(Shell))
            )
        )
}
