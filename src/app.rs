use std::ffi::OsString;
use std::io;
use std::str::FromStr;

use log::LevelFilter;

use crate::cli::build_cli;
use crate::cmd::{
    execute_completion, execute_diagram_generate, execute_library_generate, execute_library_schema,
    execute_workspace_init,
};

pub fn start_app<I, T>(args: I) -> i32
    where
        I: IntoIterator<Item=T>,
        T: Into<OsString> + Clone,
{
    let mut app = build_cli();

    let app_matches = match app.clone().try_get_matches_from(args) {
        Ok(app_matches) => app_matches,
        Err(e) => {
            eprintln!("{}", e);
            return if e.use_stderr() { 1 } else { 0 };
        }
    };
    let level_filter = match app_matches.get_one::<String>("log_level") {
        None => LevelFilter::Info,
        Some(v) => match LevelFilter::from_str(v) {
            Ok(v) => v,
            Err(_) => {
                eprintln!("unable to parse the log level: {}", v);
                return 2;
            }
        },
    };

    if let Err(e) = env_logger::builder()
        .filter_level(level_filter)
        .is_test(false)
        .try_init()
    {
        eprintln!("unable to configure the logger: {}", e);
    }

    return match app_matches.subcommand() {
        Some(("library", m)) => match m.subcommand() {
            Some(("generate", m)) => {
                return match execute_library_generate(m) {
                    Ok(_) => 0,
                    Err(e) => {
                        log::error!("the command failed: {}", e);
                        2
                    }
                };
            }
            Some(("schema", m)) => {
                return match execute_library_schema(m) {
                    Ok(_) => 0,
                    Err(e) => {
                        log::error!("the command failed: {}", e);
                        2
                    }
                };
            }
            _ => {
                log::warn!("the SUBCOMMAND is missing");
                app.write_help(&mut io::stderr())
                    .expect("unable to write help message");
                eprintln!();
                2
            }
        },
        Some(("workspace", m)) => match m.subcommand() {
            Some(("init", m)) => {
                return match execute_workspace_init(m) {
                    Ok(_) => 0,
                    Err(e) => {
                        log::error!("the command failed: {}", e);
                        2
                    }
                };
            }
            _ => {
                log::warn!("the SUBCOMMAND is missing");
                app.write_help(&mut io::stderr())
                    .expect("unable to write help message");
                eprintln!();
                2
            }
        },
        Some(("diagram", m)) => match m.subcommand() {
            Some(("generate", m)) => {
                return match execute_diagram_generate(m) {
                    Ok(_) => 0,
                    Err(e) => {
                        log::error!("the command failed: {}", e);
                        2
                    }
                };
            }
            _ => {
                log::warn!("the SUBCOMMAND is missing");
                app.write_help(&mut io::stderr())
                    .expect("unable to write help message");
                eprintln!();
                2
            }
        },
        Some(("completion", m)) => {
            return match execute_completion(m) {
                Ok(_) => 0,
                Err(e) => {
                    log::error!("the command failed: {}", e);
                    2
                }
            };
        }
        _ => {
            log::warn!("the SUBCOMMAND is missing");
            app.write_help(&mut io::stderr())
                .expect("unable to write help message");
            eprintln!();
            2
        }
    };
}
