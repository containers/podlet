use std::fmt::Display;

use clap::{Args, Subcommand};

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Network {
    /// Generate a podman quadlet `.network` file
    ///
    /// Only options supported by quadlet are present
    ///
    /// For details on options see:
    /// https://docs.podman.io/en/latest/markdown/podman-network-create.1.html and
    /// https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html#network-units-network
    #[group(skip)]
    Create {
        #[command(flatten)]
        create: Create,
    },
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self::Create { create } = self;
        writeln!(f, "[Network]")?;
        write!(f, "{create}")
    }
}

#[derive(Args, Debug, Clone, PartialEq)]
pub struct Create {
    /// The name of the network to create
    ///
    /// This will be used as the name of the generated file when used with
    /// the --file option without a filename
    name: String,
}

impl Display for Create {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
