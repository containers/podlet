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

// This binary is a thin wrapper around the `podlet` library, so it only uses a couple of crates
// directly. The remaining dependencies are used by the library target, which still enforces the
// `unused_crate_dependencies` lint.
#![allow(unused_crate_dependencies)]

use clap::Parser;
use color_eyre::eyre;

use podlet::Cli;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    Cli::parse().print_or_write_files()
}
