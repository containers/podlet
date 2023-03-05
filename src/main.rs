#![warn(clippy::pedantic)]

use clap::Parser;

mod cli;

fn main() {
    let args = cli::Cli::parse();
    println!("args:\n{args:#?}");
}
