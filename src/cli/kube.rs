use std::fmt::Display;

use clap::{Args, Subcommand};

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Kube {
    /// Generate a podman quadlet `.kube` file
    ///
    /// Only options supported by quadlet are present
    ///
    /// For details on options see:
    /// https://docs.podman.io/en/latest/markdown/podman-kube-play.1.html
    /// and https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html#kube-units-kube
    #[group(skip)]
    Play {
        #[command(flatten)]
        play: Play,
    },
}

impl Display for Kube {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self::Play { play } = self;
        writeln!(f, "[Kube]")?;
        write!(f, "{play}")
    }
}

#[derive(Args, Debug, Clone, PartialEq)]
pub struct Play {
    /// The path to the Kubernetes YAML file to use
    ///
    /// Converts to "Yaml=FILE"
    file: String,
}

impl Display for Play {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Yaml={}", self.file)?;

        Ok(())
    }
}
