//! Podlet generates [podman](https://podman.io/)
//! [quadlet](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html)
//! (systemd-like) files from a podman command.
//!
//! # Usage
//!
//! ```shell
//! $ podlet podman run quay.io/podman/hello
//! [Container]
//! Image=quay.io/podman/hello
//! ```
//!
//! Run `podlet --help` for more information.

#![warn(clippy::pedantic)]

use clap::Parser;

mod cli;

fn main() {
    let args = cli::Cli::parse();

    println!("args:\n{args:#?}");

    let cli::Commands::Podman { command } = args.command;
    print!("{command}");
}
