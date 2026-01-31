extern crate clap;

use std::env::args;
use std::process::exit;

use crate::app::start_app;

mod app;
mod cli;
mod cmd;
mod constants;
mod counter;
mod plantuml;
mod tera;
mod threading;
mod urn;
mod utils;

fn main() {
    exit(start_app(args()))
}
