mod container;
mod kube;
mod network;
mod service;
mod unit;
mod volume;

use std::{borrow::Cow, fmt::Display};

use clap::{Parser, Subcommand};

use self::{
    container::Container, kube::Kube, network::Network, service::Service, unit::Unit,
    volume::Volume,
};

#[derive(Parser, Debug, Clone, PartialEq)]
#[command(author, version, about)]
pub struct Cli {
    #[command(flatten)]
    unit: Unit,

    #[command(subcommand)]
    command: Commands,
}

impl Display for Cli {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.unit.is_empty() {
            writeln!(f, "{}", &self.unit)?;
        }

        let Commands::Podman { command } = &self.command;
        write!(f, "{command}")
    }
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

    /// Generate a podman quadlet `.volume` file
    ///
    /// For details on options see:
    /// https://docs.podman.io/en/latest/markdown/podman-volume-create.1.html
    Volume {
        #[command(subcommand)]
        volume: Volume,
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
            Self::Volume { volume } => write!(f, "{volume}"),
        }
    }
}

fn escape_spaces_join<'a>(words: impl IntoIterator<Item = &'a String>) -> String {
    words
        .into_iter()
        .map(|word| {
            if word.contains(' ') {
                format!("\"{word}\"").into()
            } else {
                word.into()
            }
        })
        .collect::<Vec<Cow<_>>>()
        .join(" ")
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
