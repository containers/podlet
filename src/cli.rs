mod container;
mod install;
mod k8s;
mod kube;
mod network;
pub mod service;
pub mod unit;
pub mod volume;

#[cfg(unix)]
mod systemd_dbus;

use std::{
    borrow::Cow,
    collections::HashMap,
    env,
    ffi::OsStr,
    fmt::{self, Display},
    fs,
    io::{self, Write},
    iter, mem,
    path::{Path, PathBuf},
    rc::Rc,
};

use clap::{Parser, Subcommand};
use color_eyre::{
    eyre::{self, Context},
    Help,
};
use docker_compose_types::{Compose, MapOrEmpty};
use k8s_openapi::api::core::v1::{PersistentVolumeClaim, Pod};

use crate::quadlet;

use self::{
    container::Container, install::Install, kube::Kube, network::Network, service::Service,
    unit::Unit, volume::Volume,
};

#[allow(clippy::option_option)]
#[derive(Parser, Debug, Clone, PartialEq)]
#[command(author, version, about, subcommand_precedence_over_arg = true)]
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

    /// Overwrite existing files when generating a file
    ///
    /// By default, podlet will return an error if a file already exists at the given location.
    #[arg(long, alias = "override", requires = "file_out")]
    overwrite: bool,

    /// Skip the check for existing services of the same name
    ///
    /// By default, podlet will check for existing services with the same name as
    /// the service quadlet will generate from the generated quadlet file
    /// and return an error if a conflict is found.
    /// This option will cause podlet to skip that check.
    #[arg(long, requires = "file_out")]
    skip_services_check: bool,

    /// The \[Unit\] section
    #[command(flatten)]
    unit: Unit,

    /// The \[Install\] section
    #[command(flatten)]
    install: Install,

    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    pub fn print_or_write_files(self) -> eyre::Result<()> {
        if self.unit_directory || self.file.is_some() {
            let path = self.file_path()?;
            if matches!(path, FilePath::Full(..))
                && matches!(self.command, Commands::Compose { .. })
            {
                return Err(eyre::eyre!(
                    "A file path was provided to `--file` and the `compose` command was used"
                )
                .suggestion(
                    "Provide a directory to `--file`. \
                        `compose` can generate multiple files so a directory is needed.",
                ));
            }

            let overwrite = self.overwrite;
            #[cfg(unix)]
            let services_check = !self.skip_services_check;

            let files = self.try_into_files()?;

            #[cfg(unix)]
            if services_check {
                check_existing(
                    files.iter().filter_map(File::quadlet_file),
                    &path,
                    overwrite,
                )?;
            }

            for file in files {
                let path: Cow<Path> = match &path {
                    FilePath::Full(path) => path.into(),
                    FilePath::Dir(path) => {
                        let mut path = path.join(file.name());
                        path.set_extension(file.extension());
                        path.into()
                    }
                };
                file.write(&path, overwrite)?;
            }

            Ok(())
        } else {
            let files = self
                .try_into_files()?
                .into_iter()
                .map(|file| format!("# {}.{}\n{file}", file.name(), file.extension()))
                .collect::<Vec<_>>()
                .join("\n---\n\n");
            print!("{files}");
            Ok(())
        }
    }

    /// Returns the file path for the generated file
    fn file_path(&self) -> eyre::Result<FilePath> {
        let path = if self.unit_directory {
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
                return Ok(FilePath::Full(path.clone()));
            }
        } else {
            env::current_dir()
                .wrap_err("File path not provided and can't access current directory")?
        };

        Ok(FilePath::Dir(path))
    }

    fn try_into_files(self) -> color_eyre::Result<Vec<File>> {
        let unit = (!self.unit.is_empty()).then_some(self.unit);
        let install = self.install.install.then(|| self.install.into());

        match self.command {
            Commands::Podman { command } => {
                let service = command.service().cloned();
                let file = quadlet::File {
                    name: self.name.unwrap_or_else(|| String::from(command.name())),
                    unit,
                    resource: command.into(),
                    service,
                    install,
                };
                Ok(vec![file.into()])
            }
            Commands::Compose { pod, compose_file } => {
                let compose = compose_from_file(&compose_file)?;

                if !compose.extensions.is_empty() {
                    eyre::bail!("extensions are not supported");
                }

                if let Some(pod_name) = pod {
                    let (pod, persistent_volume_claims) =
                        k8s::compose_try_into_pod(compose, pod_name.clone())?;

                    let kube = quadlet::Kube {
                        config_map: Vec::new(),
                        log_driver: None,
                        network: Vec::new(),
                        publish_port: Vec::new(),
                        user_ns: None,
                        yaml: format!("{pod_name}-kube.yaml"),
                    };

                    let quadlet_file = quadlet::File {
                        name: pod_name.clone(),
                        unit,
                        resource: kube.into(),
                        service: None,
                        install,
                    };

                    Ok(vec![
                        quadlet_file.into(),
                        File::KubePod {
                            name: format!("{pod_name}-kube"),
                            pod,
                            persistent_volume_claims,
                        },
                    ])
                } else {
                    compose_try_into_quadlet_files(compose, &unit, &install)
                        .map(|result| result.map(Into::into))
                        .collect()
                }
            }
        }
    }
}

#[derive(Debug)]
enum FilePath {
    Full(PathBuf),
    Dir(PathBuf),
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
enum Commands {
    /// Generate a podman quadlet file from a podman command
    Podman {
        #[command(subcommand)]
        command: PodmanCommands,
    },

    /// Generate podman quadlet files from a compose file
    ///
    /// Creates a `.container` file for each service,
    /// a `.volume` file for each volume,
    /// and a `.network` file for each network.
    ///
    /// The `--file` option must be a directory if used.
    ///
    /// Some compose options are not supported, such as `build`.
    ///
    /// When podlet encounters an unsupported option, an error will be returned.
    /// Modify the compose file to resolve the error.
    Compose {
        /// Create a Kubernetes YAML file for a pod instead of separate containers
        ///
        /// A `.kube` file using the generated Kubernetes YAML file will also be created.
        #[arg(long)]
        pod: Option<String>,

        /// The compose file to convert
        ///
        /// If not provided, podlet will look for (in order)
        /// `compose.yaml`, `compose.yml`, `docker-compose.yaml`, and `docker-compose.yml`,
        /// in the current working directory.
        compose_file: Option<PathBuf>,
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

impl TryFrom<ComposeService> for PodmanCommands {
    type Error = color_eyre::Report;

    fn try_from(value: ComposeService) -> Result<Self, Self::Error> {
        let service = (&value.service).try_into()?;
        Ok(Self::Run {
            container: Box::new(value.try_into()?),
            service,
        })
    }
}

impl From<PodmanCommands> for quadlet::Resource {
    fn from(value: PodmanCommands) -> Self {
        match value {
            PodmanCommands::Run { container, .. } => (*container).into(),
            PodmanCommands::Kube { kube } => kube.into(),
            PodmanCommands::Network { network } => network.into(),
            PodmanCommands::Volume { volume } => volume.into(),
        }
    }
}

impl PodmanCommands {
    fn service(&self) -> Option<&Service> {
        match self {
            Self::Run { service, .. } => (!service.is_empty()).then_some(service),
            _ => None,
        }
    }

    /// Returns the name that should be used for the generated file
    fn name(&self) -> &str {
        match self {
            Self::Run { container, .. } => container.name(),
            Self::Kube { kube } => kube.name(),
            Self::Network { network } => network.name(),
            Self::Volume { volume } => volume.name(),
        }
    }
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)] // false positive, [Pod] is not zero-sized
enum File {
    Quadlet(quadlet::File),
    KubePod {
        name: String,
        pod: Pod,
        persistent_volume_claims: Vec<PersistentVolumeClaim>,
    },
}

impl From<quadlet::File> for File {
    fn from(value: quadlet::File) -> Self {
        Self::Quadlet(value)
    }
}

impl Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Quadlet(file) => file.fmt(f),
            Self::KubePod {
                name: _,
                pod,
                persistent_volume_claims,
            } => {
                for volume in persistent_volume_claims {
                    f.write_str(&serde_yaml::to_string(volume).map_err(|_| fmt::Error)?)?;
                    writeln!(f, "---")?;
                }
                f.write_str(&serde_yaml::to_string(pod).map_err(|_| fmt::Error)?)
            }
        }
    }
}

impl File {
    fn name(&self) -> &str {
        match self {
            Self::Quadlet(file) => &file.name,
            Self::KubePod { name, .. } => name,
        }
    }

    fn extension(&self) -> &str {
        match self {
            Self::Quadlet(file) => file.resource.extension(),
            Self::KubePod { .. } => "yaml",
        }
    }

    fn quadlet_file(&self) -> Option<&quadlet::File> {
        match self {
            Self::Quadlet(file) => Some(file),
            Self::KubePod { .. } => None,
        }
    }

    fn write(&self, path: impl AsRef<Path>, overwrite: bool) -> color_eyre::Result<()> {
        let path_display = path.as_ref().display().to_string();
        let mut file = fs::File::options()
            .write(true)
            .create_new(!overwrite)
            .create(overwrite)
            .open(path)
            .map_err(|error| match error.kind() {
                io::ErrorKind::AlreadyExists => {
                    eyre::eyre!("File already exists, not overwriting it: {path_display}")
                        .suggestion("Use `--overwrite` if you wish overwrite existing files.")
                }
                _ => color_eyre::Report::new(error)
                    .wrap_err(format!("Failed to create/open file: {path_display}"))
                    .suggestion(
                        "Make sure the directory exists \
                                and you have write permissions for the file",
                    ),
            })?;
        match self {
            Self::Quadlet(quadlet_file) => {
                write!(file, "{quadlet_file}").map_err(color_eyre::Report::from)
            }
            Self::KubePod {
                name: _,
                pod,
                persistent_volume_claims,
            } => {
                for volume in persistent_volume_claims {
                    serde_yaml::to_writer(&file, volume)?;
                    writeln!(file, "---")?;
                }
                serde_yaml::to_writer(file, pod).map_err(color_eyre::Report::from)
            }
        }
        .wrap_err_with(|| format!("Failed to write to file: {path_display}"))?;
        println!("Wrote to file: {path_display}");
        Ok(())
    }
}

#[derive(Debug)]
struct ComposeService {
    service: docker_compose_types::Service,
    volume_has_options: Rc<HashMap<String, bool>>,
}

impl ComposeService {
    fn volume_has_options(&self, volume: &str) -> bool {
        self.volume_has_options
            .get(volume)
            .copied()
            .unwrap_or_default()
    }
}

fn compose_from_file(compose_file: &Option<PathBuf>) -> color_eyre::Result<Compose> {
    let (compose_file, path) = if let Some(path) = compose_file {
        let compose_file = fs::File::open(path)
            .wrap_err("Could not open provided compose file")
            .suggestion("Make sure you have the proper permissions for the given file.")?;
        (compose_file, path.display().to_string())
    } else {
        let file_names = [
            "compose.yaml",
            "compose.yml",
            "docker-compose.yaml",
            "docker-compose.yml",
        ];
        let mut result = None;
        for file_name in file_names {
            if let Ok(compose_file) = fs::File::open(file_name) {
                result = Some((compose_file, String::from(file_name)));
                break;
            }
        }
        result.ok_or_else(|| {
            eyre::eyre!(
                "A compose file was not provided and none of \
                `compose.yaml`, `compose.yml`, `docker-compose.yaml`, or `docker-compose.yml` \
                exist in the current directory or could not be read"
            )
        })?
    };

    serde_yaml::from_reader(compose_file)
        .wrap_err_with(|| format!("File `{path}` is not a valid compose file"))
}

fn compose_try_into_quadlet_files<'a>(
    mut compose: Compose,
    unit: &'a Option<Unit>,
    install: &'a Option<quadlet::Install>,
) -> impl Iterator<Item = color_eyre::Result<quadlet::File>> + 'a {
    let volume_has_options = compose
        .volumes
        .0
        .iter()
        .map(|(name, volume)| (name.clone(), matches!(volume, MapOrEmpty::Map(_))))
        .collect();
    compose_services(&mut compose)
        .zip(iter::repeat(Rc::new(volume_has_options)))
        .map(|(result, volume_has_options)| {
            result.and_then(|(name, mut service)| {
                let mut unit = unit.clone();
                if !service.depends_on.is_empty() {
                    unit.get_or_insert(Unit::default())
                        .add_dependencies(mem::take(&mut service.depends_on));
                }

                let service = ComposeService {
                    service,
                    volume_has_options,
                };
                let command: PodmanCommands = service.try_into().wrap_err_with(|| {
                    format!("Could not parse service `{name}` as a valid podman command")
                })?;

                let service = command.service().cloned();

                Ok(quadlet::File {
                    name,
                    unit,
                    resource: command.into(),
                    service,
                    install: install.clone(),
                })
            })
        })
        .chain(compose.networks.0.into_iter().map(|(name, network)| {
            let network = Option::<docker_compose_types::NetworkSettings>::from(network)
                .map(quadlet::Network::try_from)
                .transpose()
                .wrap_err_with(|| {
                    format!("Could not parse network `{name}` as a valid podman network")
                })?
                .unwrap_or_default();
            Ok(quadlet::File {
                name,
                unit: unit.clone(),
                resource: network.into(),
                service: None,
                install: install.clone(),
            })
        }))
        .chain(compose.volumes.0.into_iter().filter_map(|(name, volume)| {
            Option::<docker_compose_types::ComposeVolume>::from(volume).map(|volume| {
                let volume = quadlet::Volume::try_from(volume).wrap_err_with(|| {
                    format!("could not parse volume `{name}` as a valid podman volume")
                })?;
                Ok(quadlet::File {
                    name,
                    unit: unit.clone(),
                    resource: volume.into(),
                    service: None,
                    install: install.clone(),
                })
            })
        }))
}

fn compose_services(
    compose: &mut Compose,
) -> impl Iterator<Item = color_eyre::Result<(String, docker_compose_types::Service)>> {
    mem::take(&mut compose.services.0)
        .into_iter()
        .map(|(name, service)| {
            let service_name = name.clone();
            service.map(|service| (name, service)).ok_or_else(|| {
                eyre::eyre!(
                    "Service `{service_name}` does not have any corresponding options; \
                        minimally, `image` is required"
                )
            })
        })
        .chain(
            compose
                .service
                .take()
                .map(|service| Ok((String::from(image_to_name(service.image())), service))),
        )
}

/// Takes an image and returns an appropriate default service name
fn image_to_name(image: &str) -> &str {
    let image = image
        .rsplit('/')
        .next()
        .expect("Split will have at least one element");
    // Remove image tag
    image.split_once(':').map_or(image, |(name, _)| name)
}

#[cfg(unix)]
fn check_existing<'a>(
    quadlet_files: impl IntoIterator<Item = &'a quadlet::File>,
    path: &FilePath,
    overwrite: bool,
) -> eyre::Result<()> {
    if let Ok(unit_files) = systemd_dbus::unit_files().map(Iterator::collect::<Vec<_>>) {
        let file_names: Vec<_> = quadlet_files
            .into_iter()
            .filter_map(|file| match &path {
                FilePath::Full(path) => path.file_stem().and_then(OsStr::to_str).map(|name| {
                    let service = file.resource.name_to_service(name);
                    (name, service)
                }),
                FilePath::Dir(_) => Some((file.name.as_str(), file.service_name())),
            })
            .collect();
        for systemd_dbus::UnitFile { file_name, status } in unit_files {
            for (name, service) in &file_names {
                if !(overwrite && status == "generated") && file_name.contains(service) {
                    return Err(eyre::eyre!(
                        "File name `{name}` conflicts with existing unit file: {file_name}"
                    )
                    .suggestion(
                        "Change the generated file's name with `--file` or `--name`. \
                                Alternatively, use `--skip-services-check` if this is ok.",
                    ));
                }
            }
        }
    }

    Ok(())
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
