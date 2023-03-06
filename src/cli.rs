use std::fmt::Display;

use clap::{Parser, Subcommand};

mod container;

#[derive(Parser, Debug, Clone, PartialEq)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Commands {
    /// Generate a podman quadlet file from a podman command
    Podman {
        #[command(subcommand)]
        command: PodmanCommands,
    },
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum PodmanCommands {
    /// Generate a podman quadlet `.container` file
    ///
    /// For details on options see:
    /// https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html
    Run {
        #[command(flatten)]
        quadlet_options: container::QuadletOptions,

        /// Converts to "PodmanArgs=ARGS"
        #[command(flatten)]
        podman_args: container::PodmanArgs,

        /// The image to run in the container
        ///
        /// Converts to "Image=IMAGE"
        image: String,

        /// Optionally, the command to run in the container
        ///
        /// Converts to "Exec=COMMAND..."
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,
    },
}

impl Display for PodmanCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PodmanCommands::Run {
                quadlet_options,
                podman_args,
                image,
                command,
            } => {
                writeln!(f, "[Container]")?;
                writeln!(f, "Image={image}")?;
                write!(f, "{quadlet_options}")?;
                write!(f, "{podman_args}")?;
                if !command.is_empty() {
                    let command = shlex::join(command.iter().map(String::as_str));
                    writeln!(f, "Exec={command}")?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
