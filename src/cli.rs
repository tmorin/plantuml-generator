use clap::{crate_authors, crate_description, crate_version, App, AppSettings, Arg, SubCommand};

pub fn build_cli() -> App<'static, 'static> {
    let arg_cache_directory: Arg = Arg::with_name("cache_directory")
        .short("C")
        .long("cache")
        .env("PLANTUML_GENERATOR_OUTPUT_CACHE")
        .help("The cache directory.");

    let arg_plantuml_version: Arg = Arg::with_name("plantuml_version")
        .conflicts_with("plantuml_jar")
        .short("V")
        .long("plantuml-version")
        .env("PLANTUML_GENERATOR_PLANTUML_VERSION")
        .help("The PlantUML version.");

    let arg_plantuml_jar: Arg = Arg::with_name("plantuml_jar")
        .conflicts_with("plantuml_version")
        .short("P")
        .long("plantuml")
        .env("PLANTUML_GENERATOR_PLANTUML_JAR")
        .help("The PlantUML version.");

    let arg_java_binary: Arg = Arg::with_name("java_binary")
        .short("J")
        .long("java")
        .env("PLANTUML_GENERATOR_JAVA_BINARY")
        .help("The java binary path or command line.");

    let arg_inkscape_binary: Arg = Arg::with_name("inkscape_binary")
        .short("I")
        .long("inkscape")
        .env("PLANTUML_GENERATOR_INKSCAPE_BINARY")
        .help("The inkscape binary path or command line.");

    App::new("plantuml-generator")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("log_level")
                .short("l")
                .long("log-level")
                .takes_value(true)
                .default_value("Info")
                .possible_values(&["Off", "Trace", "Debug", "Info", "Warn", "Error"])
                .help("Set the verbosity of the logs."),
        )
        .subcommand(
            SubCommand::with_name("library")
                .about("Manage libraries")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name("generate")
                        .about("Generate a library from a manifest.")
                        .arg(Arg::with_name("MANIFEST")
                            .index(1)
                            .required(true)
                            .takes_value(true)
                            .help("The manifest of the library."))
                        .arg(Arg::with_name("output_directory")
                            .short("O")
                            .long("output")
                            .env("PLANTUML_GENERATOR_OUTPUT_DIRECTORY")
                            .help("The output directory."))
                        .arg(Arg::with_name("urns")
                            .short("u")
                            .long("urn")
                            .takes_value(true)
                            .multiple(true)
                            .help("Handle only artifacts included in the URN."))
                        .arg(Arg::with_name("do_clean_cache")
                            .long("clean-cache")
                            .help("Delete the cache directory before the generation-"))
                        .arg(Arg::with_name("urns_to_clean")
                            .help("Delete the given URN in the output directory before the generation.")
                            .long("clean-urn")
                            .takes_value(true)
                            .multiple(true))
                        .arg(Arg::with_name("cleanup_scopes")
                            .help("The scopes to cleanup before the generation.")
                            .long_help("By default, artifacts which are already generated won't be generated again. The cleanup-scope option helps to target artifacts which will be re-generated.")
                            .short("c")
                            .long("cleanup-scope")
                            .takes_value(true)
                            .multiple(true)
                            .possible_values(&[
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
                        .arg(&arg_cache_directory)
                        .arg(&arg_plantuml_version)
                        .arg(&arg_plantuml_jar)
                        .arg(&arg_java_binary)
                        .arg(&arg_inkscape_binary),
                ),
        )
        .subcommand(
            SubCommand::with_name("diagram")
                .about("Manage diagrams")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name("generate")
                        .about("Generate discovered .puml files which has been mutated since the last generation.")
                        .arg(Arg::with_name("source_directory")
                            .short("s")
                            .long("source")
                            .default_value(".")
                            .env("PLANTUML_GENERATOR_SOURCE_DIRECTORY")
                            .help("The directory where the .puml will be discovered."))
                        .arg(Arg::with_name("do_force_generation")
                            .short("f")
                            .long("force")
                            .help("Force the rendering of discovered .puml file."))
                        .arg(&arg_cache_directory)
                        .arg(&arg_plantuml_version)
                        .arg(&arg_plantuml_jar)
                        .arg(&arg_java_binary)
                ),
        )
}
