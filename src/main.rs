use clap::Parser;
use std::process;

use org2typst::{run, Config};

fn main() {
    let config = Config::parse();

    if let Err(e) = run(config) {
        eprintln!("Problem converting file: {e}");
        process::exit(1);
    }
}
