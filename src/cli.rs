mod container;
mod kube;
mod network;
mod service;
mod unit;
mod volume;

use std::{
    borrow::Cow,
    env,
    fmt::Display,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use color_eyre::eyre::{self, Context};

use self::{
    container::Container, kube::Kube, network::Network, service::Service, unit::Unit,
    volume::Volume,
};

#[allow(clippy::option_option)]
#[derive(Parser, Debug, Clone, PartialEq)]
#[command(author, version, about)]
pub struct Cli {
    /// Generate a file instead of printing to stdout
    ///
    /// Optionally provide a path for the file,
    /// if no path is provided the file will be placed in the current working directory.
    /// If not provided, the name of the generated file will be taken from,
    /// the `name` parameter for volumes and networks,
    /// the filename of the kube file,
    /// the container name,
    /// or the name of the container image.
    #[arg(short, long)]
    file: Option<Option<PathBuf>>,

    /// Override the name of the generated file (without the extension)
    ///
    /// This only applies if a file was not given to the --file option.
    ///
    /// E.g. `podlet --file --name hello-world podman run quay.io/podman/hello`
    /// will generate a file with the name "hello-world.container".
    #[arg(short, long, requires = "file")]
    name: Option<String>,

    /// The \[Unit\] section
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

impl Cli {
    pub fn print_or_write_file(&self) -> eyre::Result<()> {
        if self.file.is_some() {
            let path = self.file_path()?;
            let mut file = File::create(&path)?;
            write!(file, "{self}").wrap_err("Failed to write to file")?;
            println!("Wrote to file: {}", path.display());
            Ok(())
        } else {
            print!("{self}");
            Ok(())
        }
    }

    /// Returns the file path for the generated file
    fn file_path(&self) -> eyre::Result<Cow<Path>> {
        let mut path = if let Some(Some(path)) = &self.file {
            if path.is_dir() {
                path.clone()
            } else {
                return Ok(path.into());
            }
        } else {
            env::current_dir()
                .wrap_err("File path not provided and can't access current directory")?
        };

        let Commands::Podman { command } = &self.command;

        path.push(self.name.as_deref().unwrap_or_else(|| command.name()));
        path.set_extension(command.extension());

        Ok(path.into())
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
        /// The \[Kube\] section
        #[command(subcommand)]
        kube: Kube,
    },

    /// Generate a podman quadlet `.network` file
    ///
    /// For details on options see:
    /// https://docs.podman.io/en/latest/markdown/podman-network-create.1.html
    Network {
        /// The \[Network\] section
        #[command(subcommand)]
        network: Network,
    },

    /// Generate a podman quadlet `.volume` file
    ///
    /// For details on options see:
    /// https://docs.podman.io/en/latest/markdown/podman-volume-create.1.html
    Volume {
        /// The \[Volume\] section
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

impl PodmanCommands {
    /// Returns the name that should be used for the generated file
    fn name(&self) -> &str {
        match self {
            PodmanCommands::Run { container, .. } => container.name(),
            PodmanCommands::Kube { kube } => kube.name(),
            PodmanCommands::Network { network } => network.name(),
            PodmanCommands::Volume { volume } => volume.name(),
        }
    }

    /// Returns the extension that should be used for the generated file
    fn extension(&self) -> &'static str {
        match self {
            PodmanCommands::Run { .. } => "container",
            PodmanCommands::Kube { .. } => "kube",
            PodmanCommands::Network { .. } => "network",
            PodmanCommands::Volume { .. } => "volume",
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
