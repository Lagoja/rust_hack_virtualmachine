extern crate regex;

use std::env;
use std::process;
use lib::vm;

mod lib;

fn main() {
    let config = vm::Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Could not parse file {}", err);
        process::exit(1);
    });

    if let Err(e) = vm::run(config) {
        eprintln!("Application Error: {}", e);
        process::exit(1);
    }
}
