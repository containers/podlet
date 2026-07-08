//! Podlet generates [Podman](https://podman.io/)
//! [Quadlet](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html)
//! (systemd-like) files from a Podman command, compose file, or existing object.
//!
//! This crate can be used both as the `podlet` command-line application and as a library.
//!
//! # Library usage
//!
//! The most common library use case is converting a compose file into Quadlet files entirely
//! in memory, without touching the filesystem. This makes it suitable for use in other
//! applications, including WebAssembly (WASM) targets running in the browser.
//!
//! ```
//! use podlet::{compose_to_files, ComposeOptions};
//!
//! let compose = "\
//! services:
//!   caddy:
//!     image: docker.io/library/caddy:latest
//!     ports:
//!       - 8000:80
//! ";
//!
//! let files = compose_to_files(compose, ComposeOptions::default()).unwrap();
//! assert_eq!(files[0].name, "caddy.container");
//! assert!(files[0].content.contains("Image=docker.io/library/caddy:latest"));
//! ```
//!
//! For full control (equivalent to the CLI, including the `podman ...` and `generate`
//! subcommands), construct a [`Cli`] and use [`Cli::try_into_generated_files`] to obtain the
//! generated files in memory instead of printing or writing them.
//!
//! # Command-line usage
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

use std::collections::HashSet;

use color_eyre::eyre::WrapErr;
use compose_spec::Compose;

pub use self::cli::Cli;
use self::{
    cli::{File, compose::Compose as ComposeCommand},
    quadlet::{GenericSections, JoinOption},
};

/// A generated output file, held entirely in memory.
///
/// Returned by the in-memory conversion helpers such as [`compose_to_files`]. This is the
/// building block for consumers that do not want Podlet to write to the filesystem itself
/// (e.g. a web frontend compiled to WASM).
#[derive(Debug, Clone, PartialEq, Eq, ::serde::Serialize)]
pub struct GeneratedFile {
    /// The file name, including its extension (e.g. `caddy.container` or `caddy-kube.yaml`).
    pub name: String,

    /// The serialized contents of the file.
    pub content: String,
}

/// Options controlling how a compose file is converted into Quadlet files.
///
/// The defaults match `podlet compose` with no additional flags.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ComposeOptions {
    /// Create a `.pod` file and link it with each `.container` file.
    ///
    /// The top-level `name` field in the compose file is required when this is set.
    pub pod: bool,

    /// Create a Kubernetes YAML file for a pod instead of separate containers.
    ///
    /// The top-level `name` field in the compose file is required when this is set.
    pub kube: bool,

    /// Set `ContainerName=` for each container to the compose service name.
    pub add_container_name: bool,

    /// Quadlet options to split onto separate lines instead of joining them together.
    ///
    /// When empty (the default), all joinable options are combined onto a single line, matching
    /// the CLI's default behavior.
    pub split_options: Vec<JoinOption>,
}

/// Convert the YAML contents of a compose file into generated Quadlet (and, optionally,
/// Kubernetes) files, entirely in memory.
///
/// This performs no filesystem, environment, or process access, making it usable on restricted
/// targets such as `wasm32-unknown-unknown`.
///
/// # Errors
///
/// Returns an error if the compose file cannot be parsed or validated, or if it cannot be
/// converted into Quadlet files (for example, when it uses an unsupported option).
pub fn compose_to_files(
    yaml: &str,
    options: ComposeOptions,
) -> color_eyre::Result<Vec<GeneratedFile>> {
    let ComposeOptions {
        pod,
        kube,
        add_container_name,
        split_options,
    } = options;

    let mut parse_options = Compose::options();
    parse_options.apply_merge(true);
    let compose = parse_options
        .from_yaml_reader(yaml.as_bytes())
        .wrap_err("input is not a valid compose file")?;

    let command = ComposeCommand {
        pod,
        kube,
        add_container_name,
        compose_file: None,
    };

    let files = command
        .into_files(compose, GenericSections::default())
        .wrap_err("error converting compose file")?;

    let join_options = &JoinOption::all_set() - &split_options.into_iter().collect();
    serialize_files(&files, &join_options)
}

/// Serialize in-memory [`File`]s into [`GeneratedFile`]s.
///
/// Quadlet options in `join_options` are combined onto a single line.
///
/// # Errors
///
/// Returns an error if any file fails to serialize.
pub(crate) fn serialize_files(
    files: &[File],
    join_options: &HashSet<JoinOption>,
) -> color_eyre::Result<Vec<GeneratedFile>> {
    files
        .iter()
        .map(|file| {
            let name = format!("{}.{}", file.name(), file.extension());
            let content = file
                .serialize(join_options)
                .wrap_err_with(|| format!("error serializing file `{name}`"))?;
            Ok(GeneratedFile { name, content })
        })
        .collect()
}
