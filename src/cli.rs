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
    Run {
        #[command(flatten)]
        quadlet_options: container::QuadletOptions,
        #[command(flatten)]
        podman_args: container::PodmanArgs,
        image: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command: Option<Vec<String>>,
    },
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
