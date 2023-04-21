mod container;
mod install;
mod kube;
mod network;
mod service;
mod unit;
mod volume;

#[cfg(unix)]
mod systemd_dbus;

use std::{
    borrow::Cow,
    env,
    ffi::OsStr,
    fmt::Display,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use color_eyre::{
    eyre::{self, Context},
    Help,
};

use self::{
    container::Container, install::Install, kube::Kube, network::Network, service::Service,
    unit::Unit, volume::Volume,
};

#[allow(clippy::option_option)]
#[derive(Parser, Debug, Clone, PartialEq)]
#[command(author, version, about)]
pub struct Cli {
    /// Generate a file instead of printing to stdout
    ///
    /// Optionally provide a path for the file,
    /// if no path is provided the file will be placed in the current working directory.
    ///
    /// If not provided, the name of the generated file will be taken from,
    /// the `name` parameter for volumes and networks,
    /// the filename of the kube file,
    /// the container name,
    /// or the name of the container image.
    #[arg(short, long, group = "file_out")]
    file: Option<Option<PathBuf>>,

    /// Generate a file in the podman unit directory instead of printing to stdout
    ///
    /// Conflicts with the --file option
    ///
    /// Equivalent to `--file $XDG_CONFIG_HOME/containers/systemd/` for non-root users,
    /// or `--file /etc/containers/systemd/` for root.
    ///
    /// The name of the file can be specified with the --name option.
    #[arg(
        short,
        long,
        visible_alias = "unit-dir",
        conflicts_with = "file",
        group = "file_out"
    )]
    unit_directory: bool,

    /// Override the name of the generated file (without the extension)
    ///
    /// This only applies if a file was not given to the --file option,
    /// or the --unit-directory option was used.
    ///
    /// E.g. `podlet --file --name hello-world podman run quay.io/podman/hello`
    /// will generate a file with the name "hello-world.container".
    #[arg(short, long, requires = "file_out")]
    name: Option<String>,

    /// Skip the check for existing services of the same name
    ///
    /// By default, podlet will check for existing services with the same name as
    /// the service quadlet will generate from the generated quadlet file
    /// and return an error if a conflict is found.
    /// This option will cause podlet to skip that check.
    #[arg(long, requires = "file_out")]
    skip_check_existing: bool,

    /// The \[Unit\] section
    #[command(flatten)]
    unit: Unit,

    /// The \[Install\] section
    #[command(flatten)]
    install: Install,

    #[command(subcommand)]
    command: Commands,
}

impl Display for Cli {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.unit.is_empty() {
            writeln!(f, "{}", &self.unit)?;
        }

        let Commands::Podman { command } = &self.command;
        write!(f, "{command}")?;

        if self.install.install {
            write!(f, "\n{}", &self.install)?;
        }

        Ok(())
    }
}

impl Cli {
    pub fn print_or_write_file(&self) -> eyre::Result<()> {
        if self.unit_directory || self.file.is_some() {
            let path = self.file_path()?;
            let mut file = File::create(&path)
                .wrap_err_with(|| format!("Failed to create/open file: {}", path.display()))
                .suggestion(
                    "Make sure the directory exists and you have write permissions for the file",
                )?;
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
        let mut path = if self.unit_directory {
            #[cfg(unix)]
            if nix::unistd::Uid::current().is_root() {
                let path = PathBuf::from("/etc/containers/systemd/");
                if path.is_dir() {
                    path
                } else {
                    PathBuf::from("/usr/share/containers/systemd/")
                }
            } else {
                let mut path: PathBuf = env::var("XDG_CONFIG_HOME")
                    .or_else(|_| env::var("HOME").map(|home| format!("{home}/.config")))
                    .unwrap_or_else(|_| String::from("~/.config/"))
                    .into();
                path.push("containers/systemd/");
                path
            }

            #[cfg(not(unix))]
            return Err(eyre::eyre!(
                "Cannot get podman unit directory on non-unix system"
            ));
        } else if let Some(Some(path)) = &self.file {
            if path.is_dir() {
                path.clone()
            } else {
                if let Some(name) = path.file_stem().and_then(OsStr::to_str) {
                    self.check_existing(name)?;
                }
                return Ok(path.into());
            }
        } else {
            env::current_dir()
                .wrap_err("File path not provided and can't access current directory")?
        };

        let Commands::Podman { command } = &self.command;
        let name = self.name.as_deref().unwrap_or_else(|| command.name());
        self.check_existing(name)?;

        path.push(name);
        path.set_extension(command.extension());

        Ok(path.into())
    }

    fn check_existing(&self, name: &str) -> eyre::Result<()> {
        #[cfg(unix)]
        if !self.skip_check_existing {
            if let Ok(unit_files) = systemd_dbus::unit_files() {
                let Commands::Podman { command } = &self.command;
                let service = command.name_to_service(name);
                for systemd_dbus::UnitFile { file_name, status } in unit_files {
                    if file_name.contains(&service) {
                        return Err(eyre::eyre!(
                            "File name `{name}` conflicts with existing unit file: {file_name}"
                        )
                        .suggestion(
                            "Change the generated file's name with `--file` or `--name`. \
                                Alternatively, use the `--skip-check-existing` option if this is ok",
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
enum Commands {
    /// Generate a podman quadlet file from a podman command
    Podman {
        #[command(subcommand)]
        command: PodmanCommands,
    },
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
enum PodmanCommands {
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
            Self::Run { container, .. } => container.name(),
            Self::Kube { kube } => kube.name(),
            Self::Network { network } => network.name(),
            Self::Volume { volume } => volume.name(),
        }
    }

    /// Takes a file name (no extension) and returns the corresponding service file name
    /// generated by quadlet
    fn name_to_service(&self, name: &str) -> String {
        let mut service = match self {
            Self::Run { .. } | Self::Kube { .. } => String::from(name),
            Self::Network { .. } => format!("{name}-network"),
            Self::Volume { .. } => format!("{name}-volume"),
        };
        service.push_str(".service");
        service
    }

    /// Returns the extension that should be used for the generated file
    fn extension(&self) -> &'static str {
        match self {
            Self::Run { .. } => "container",
            Self::Kube { .. } => "kube",
            Self::Network { .. } => "network",
            Self::Volume { .. } => "volume",
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
