mod container;
mod kube;
mod network;
mod service;

use std::fmt::Display;

use clap::{Parser, Subcommand};

use self::{container::Container, kube::Kube, network::Network, service::Service};

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
        /// The \[Container\] section
        #[command(flatten)]
        container: Box<Container>,

        /// The \[Service\] section
        #[command(flatten)]
        service: Service,
    },

    /// Generate a podman quadlet `.kube` file
    ///
    /// For details on options see:
    /// https://docs.podman.io/en/latest/markdown/podman-kube-play.1.html
    Kube {
        #[command(subcommand)]
        kube: Kube,
    },

    /// Generate a podman quadlet `.network` file
    ///
    /// For details on options see:
    /// https://docs.podman.io/en/latest/markdown/podman-network-create.1.html
    Network {
        #[command(subcommand)]
        network: Network,
    },
}

impl Display for PodmanCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Run { container, service } => {
                write!(f, "{container}")?;
                if !service.is_empty() {
                    write!(f, "\n{service}")?;
                }
                Ok(())
            }
            Self::Kube { kube } => write!(f, "{kube}"),
            Self::Network { network } => write!(f, "{network}"),
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
