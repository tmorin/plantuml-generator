use std::ffi::OsString;
use std::str::FromStr;

use crate::cli::build_cli;

use crate::cmd::execute_diagram_generate;
use crate::cmd::execute_library_generate;
use log::LevelFilter;

pub fn start_app<I, T>(args: I) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let app = build_cli();

    let app_matches = match app.clone().get_matches_from_safe(args) {
        Ok(app_matches) => app_matches,
        Err(e) => {
            return if e.use_stderr() {
                eprintln!("{}", e.message);
                1
            } else {
                eprintln!("{}", e.message);
                0
            }
        }
    };

    let level_filter = match app_matches.value_of("log_level") {
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
        ("library", Some(m)) => match m.subcommand() {
            ("generate", Some(m)) => {
                return match execute_library_generate(m) {
                    Ok(_) => 0,
                    Err(e) => {
                        log::error!("the command failed: {}", e);
                        2
                    }
                };
            }
            _ => {
                log::warn!("the SUBCOMMAND is missing");
                app.clone()
                    .write_help(&mut std::io::stderr())
                    .expect("unable to write help message");
                eprintln!();
                2
            }
        },
        ("diagram", Some(m)) => match m.subcommand() {
            ("generate", Some(m)) => {
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
                app.clone()
                    .write_help(&mut std::io::stderr())
                    .expect("unable to write help message");
                eprintln!();
                2
            }
        },
        _ => {
            log::warn!("the SUBCOMMAND is missing");
            app.clone()
                .write_help(&mut std::io::stderr())
                .expect("unable to write help message");
            eprintln!();
            2
        }
    };
}
