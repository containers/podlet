//! Podlet generates [Podman](https://podman.io/)
//! [Quadlet](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html)
//! (systemd-like) files from a Podman command, compose file, or existing object.
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

mod cli;
mod escape;
mod quadlet;
mod serde;

use clap::Parser;
use color_eyre::eyre;

use self::cli::Cli;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    Cli::parse().print_or_write_files()
}
