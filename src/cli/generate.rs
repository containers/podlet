//! Provides the `podlet generate` subcommand, see [`Generate`].
//!
//! `podlet generate` uses the podman `inspect` commands to get information on the selected
//! resource. The information is converted into a [`PodmanCommands`] which, in turn, is turned into
//! a [`crate::quadlet::File`].

use std::process::Command;

use clap::{Parser, Subcommand};
use color_eyre::{
    eyre::{eyre, WrapErr},
    Section, SectionExt,
};
use serde::Deserialize;

use super::{container::Container, service::Service, PodmanCommands};

/// [`Subcommand`] for `podlet generate`
#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Generate {
    /// Generate a quadlet file from an existing container
    ///
    /// The command used to create the container is parsed to generate the quadlet file.
    Container {
        /// Name of the container
        container: String,
    },
}

impl TryFrom<Generate> for PodmanCommands {
    type Error = color_eyre::Report;

    fn try_from(value: Generate) -> Result<Self, Self::Error> {
        match value {
            Generate::Container { container } => {
                ContainerParser::from_container(&container).map(Into::into)
            }
        }
    }
}

/// [`Parser`] for container creation CLI options.
#[derive(Parser, Debug)]
#[command(no_binary_name = true)]
struct ContainerParser {
    /// The \[Container\] section
    #[command(flatten)]
    container: Container,
    /// The \[Service\] section
    #[command(flatten)]
    service: Service,
}

impl ContainerParser {
    /// Runs `podman container inspect` on the container and parses the create command.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error getting the create command,
    /// or if it cannot be successfully parsed into container creation CLI options.
    fn from_container(container: &str) -> color_eyre::Result<Self> {
        let create_command = ContainerInspect::from_container(container)
            .wrap_err_with(|| {
                format!("error getting command used to create container: {container}")
            })?
            .config
            .create_command;

        Self::try_parse_from(strip_container_create_command_prefix(&create_command)).wrap_err_with(
            || {
                format!(
                    "error parsing podman command from: {}",
                    shlex::join(create_command.iter().map(String::as_str))
                )
            },
        )
    }
}

/// Remove the command part of `command`, leaving just the container creation options.
fn strip_container_create_command_prefix(command: &[String]) -> impl Iterator<Item = &String> {
    let mut iter = command.iter().peekable();

    // remove arg0, i.e. "podman" or "/usr/bin/podman"
    iter.next();

    // command could be `podman run`, `podman create`, or `podman container create`
    if iter.peek().is_some_and(|arg| *arg == "container") {
        iter.next();
    }
    if iter
        .peek()
        .is_some_and(|arg| *arg == "run" || *arg == "create")
    {
        iter.next();
    }

    iter
}

impl From<ContainerParser> for PodmanCommands {
    fn from(ContainerParser { container, service }: ContainerParser) -> Self {
        PodmanCommands::Run {
            container: Box::new(container),
            service,
        }
    }
}

/// Selected output of `podman container inspect`.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct ContainerInspect {
    config: ContainerConfig,
}

/// Part of `Config` object from the output of `podman container inspect`
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct ContainerConfig {
    create_command: Vec<String>,
}

impl ContainerInspect {
    /// Runs `podman container inspect` on the container and deserializes the output into [`Self`].
    ///
    /// # Errors
    ///
    /// Returns an error if there is problem running `podman container inspect`,
    /// it doesn't complete successfully,
    /// or if the output cannot be properly deserialized.
    fn from_container(container: &str) -> color_eyre::Result<Self> {
        let output = Command::new("podman")
            .args(["container", "inspect", container])
            .output()
            .wrap_err_with(|| format!("error running `podman container inspect {container}`"))
            .note("ensure podman is installed and available on $PATH")?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return if let Some(code) = output.status.code() {
                Err(eyre!(
                    "`podman container inspect {container}` \
                        exited unsuccessfully with status code: {code}"
                ))
            } else {
                Err(eyre!(
                    "`podman container inspect {container}` \
                        was terminated by a signal"
                ))
            }
            .section(stdout.trim().to_owned().header("Podman Stdout:"))
            .section(stderr.trim().to_owned().header("Podman Stderr:"));
        }

        // `podman container inspect` returns a JSON array which is also valid YAML so serde_yaml can
        // be reused. There should only be a single object in the array, so the first one is returned.
        serde_yaml::from_str::<Vec<Self>>(&stdout)
            .wrap_err(
                "error deserializing container create command \
                        from `podman container inspect` output",
            )
            .with_section(|| stdout.trim().to_owned().header("Podman Stdout:"))?
            .into_iter()
            .next()
            .ok_or_else(|| eyre!("no containers matching `{container}`"))
    }
}
