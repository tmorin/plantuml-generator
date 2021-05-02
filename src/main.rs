#[macro_use]
extern crate clap;

use crate::app::start_app;
use std::env::args;
use std::process::exit;

mod app;
mod cli;
mod cmd;
mod constants;
mod counter;
mod error;
mod manifest;
mod plantuml;
mod result;
mod tera;
mod urn;
mod utils;

fn main() {
    exit(start_app(args()))
}
