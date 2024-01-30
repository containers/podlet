mod compose;
mod container;
mod generate;
mod global_args;
mod image;
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
    path::{Path, PathBuf},
    rc::Rc,
};

use clap::{Parser, Subcommand};
use color_eyre::{
    eyre::{self, Context},
    Help,
};
use k8s_openapi::api::core::v1::{PersistentVolumeClaim, Pod};
use path_clean::PathClean;

use crate::quadlet::{self, Downgrade, DowngradeError, Globals, HostPaths, PodmanVersion};

use self::{
    container::Container, generate::Generate, global_args::GlobalArgs, image::Image,
    install::Install, kube::Kube, network::Network, service::Service, unit::Unit, volume::Volume,
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

    /// Podman version generated quadlet files should conform to
    ///
    /// An error will occur if the quadlet file cannot be downgraded to the given version.
    ///
    /// Always defaults to the latest supported podman version which added quadlet features.
    /// If an earlier version is specified, the quadlet file may not be the most optimal.
    ///
    /// This feature is only supported in a limited way. You should always check quadlet files
    /// generated with podlet before running them.
    #[arg(short, long, visible_aliases = ["compatibility", "compat"], default_value_t)]
    podman_version: PodmanVersion,

    /// Convert relative host paths to absolute paths.
    ///
    /// Relative host paths in generated quadlet files are resolved using the given directory or the
    /// current working directory. For `podlet compose`, the parent directory of the compose
    /// file is used as the default if the compose file is not read from stdin.
    ///
    /// All host paths are also cleaned to remove interior `/../`, `/./`, and `//`.
    ///
    /// When using `podlet compose --pod`, modifying paths in generated Kubernetes YAML files is not
    /// supported.
    ///
    /// Note that only host paths not in the `PodmanArgs=` quadlet option will be modified.
    ///
    /// Podlet will return an error if the current working directory cannot be read, or if the given
    /// directory path is not absolute.
    #[arg(short, long, value_name = "RESOLVE_DIR")]
    absolute_host_paths: Option<Option<PathBuf>>,

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
            eyre::bail!("Cannot get podman unit directory on non-unix system");
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

    /// Take the directory to resolve relative paths with.
    ///
    /// Returns [`None`] if relative paths should not be resolved.
    ///
    /// # Errors
    ///
    /// Returns an error if the resolve directory is not absolute or, when needed, the current
    /// working directory could not be read.
    fn resolve_dir(&mut self) -> color_eyre::Result<Option<PathBuf>> {
        const CURRENT_DIR_ERR: &str = "current working directory could not be read";

        self.absolute_host_paths
            .take()
            .map(|path| {
                if let Some(path) = path {
                    eyre::ensure!(
                        path.is_absolute(),
                        "path `{}` is not absolute",
                        path.display()
                    );
                    Ok(path)
                } else {
                    match &self.command {
                        Commands::Compose {
                            compose_file: Some(path),
                            ..
                        } if path.as_os_str() != "-" && !path.as_os_str().is_empty() => {
                            if let Some(path) = path.parent() {
                                let current_dir = env::current_dir().wrap_err(CURRENT_DIR_ERR)?;
                                Ok(absolute_clean_path(&current_dir, path))
                            } else {
                                // path is the root directory
                                Ok(path.to_owned())
                            }
                        }
                        _ => env::current_dir().wrap_err(CURRENT_DIR_ERR),
                    }
                }
            })
            .transpose()
    }

    /// Convert into [`File`]s
    fn try_into_files(mut self) -> color_eyre::Result<Vec<File>> {
        let resolve_dir = self
            .resolve_dir()
            .wrap_err("error with `--absolute-host-paths` resolve directory")?;

        let unit = (!self.unit.is_empty()).then_some(self.unit);
        let install = self.install.install.then(|| self.install.into());

        let mut files = self.command.try_into_files(self.name, unit, install)?;

        let downgrade = self.podman_version < PodmanVersion::LATEST;
        if downgrade || resolve_dir.is_some() {
            for file in &mut files {
                if let Some(resolve_dir) = &resolve_dir {
                    file.absolutize_host_paths(resolve_dir);
                }

                if downgrade {
                    file.downgrade(self.podman_version).wrap_err_with(|| {
                        format!(
                            "error downgrading quadlet to podman v{}",
                            self.podman_version
                        )
                    })?;
                }
            }
        }

        Ok(files)
    }
}

/// [`PathBuf`] pointing to a file or directory
#[derive(Debug)]
enum FilePath {
    /// [`PathBuf`] pointing to a file
    Full(PathBuf),
    /// [`PathBuf`] pointing to a directory
    Dir(PathBuf),
}

impl FilePath {
    /// Convert to full file path
    ///
    /// If `self` is a directory, the [`File`] is used to set the filename.
    fn to_full(&self, file: &File) -> Cow<Path> {
        match self {
            Self::Full(path) => path.into(),
            Self::Dir(path) => {
                let mut path = path.join(file.name());
                path.set_extension(file.extension());
                path.into()
            }
        }
    }
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
enum Commands {
    /// Generate a podman quadlet file from a podman command
    Podman {
        #[command(flatten)]
        global_args: Box<GlobalArgs>,

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
        /// If `-` or not provided and stdin is not a terminal,
        /// the compose file will be read from stdin.
        ///
        /// If not provided, and stdin is a terminal, podlet will look for (in order)
        /// `compose.yaml`, `compose.yml`, `docker-compose.yaml`, and `docker-compose.yml`,
        /// in the current working directory.
        compose_file: Option<PathBuf>,
    },

    /// Generate a podman quadlet file from an existing container, network, volume, or image.
    ///
    /// Note: these commands require that podman is installed and is searchable
    /// from the `PATH` environment variable.
    #[command(subcommand)]
    Generate(Generate),
}

impl Commands {
    /// Convert into [`File`]s
    ///
    /// # Errors
    ///
    /// Returns an error if there was an error reading the compose file,
    /// converting the compose file to a podman command,
    /// or getting the podman command from the existing resource.
    fn try_into_files(
        self,
        name: Option<String>,
        unit: Option<Unit>,
        install: Option<quadlet::Install>,
    ) -> color_eyre::Result<Vec<File>> {
        match self {
            Self::Podman {
                global_args,
                command,
            } => Ok(vec![command
                .into_quadlet(name, unit, (*global_args).into(), install)
                .into()]),
            Self::Compose { pod, compose_file } => {
                let compose = compose::from_file_or_stdin(compose_file.as_deref())?;

                eyre::ensure!(
                    compose.extensions.is_empty(),
                    "extensions are not supported"
                );

                if let Some(pod_name) = pod {
                    let (pod, persistent_volume_claims) =
                        k8s::compose_try_into_pod(compose, pod_name.clone())?;

                    let kube_file_name = format!("{pod_name}-kube");
                    let kube = quadlet::Kube::new(format!("{kube_file_name}.yaml").into());

                    let quadlet_file = quadlet::File {
                        name: pod_name,
                        unit,
                        resource: kube.into(),
                        globals: Globals::default(),
                        service: None,
                        install,
                    };

                    Ok(vec![
                        quadlet_file.into(),
                        File::KubePod {
                            name: kube_file_name,
                            pod,
                            persistent_volume_claims,
                        },
                    ])
                } else {
                    compose::try_into_quadlet_files(compose, unit.as_ref(), install.as_ref())
                        .map(|result| result.map(Into::into))
                        .collect()
                }
            }
            Self::Generate(command) => {
                Ok(vec![command.try_into_quadlet(name, unit, install)?.into()])
            }
        }
    }
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
enum PodmanCommands {
    /// Generate a podman quadlet `.container` file
    ///
    /// For details on options see:
    /// https://docs.podman.io/en/stable/markdown/podman-systemd.unit.5.html
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
    /// https://docs.podman.io/en/stable/markdown/podman-kube-play.1.html
    Kube {
        /// The \[Kube\] section
        #[command(subcommand)]
        kube: Box<Kube>,
    },

    /// Generate a podman quadlet `.network` file
    ///
    /// For details on options see:
    /// https://docs.podman.io/en/stable/markdown/podman-network-create.1.html
    Network {
        /// The \[Network\] section
        #[command(subcommand)]
        network: Box<Network>,
    },

    /// Generate a podman quadlet `.volume` file
    ///
    /// For details on options see:
    /// https://docs.podman.io/en/stable/markdown/podman-volume-create.1.html
    Volume {
        /// The \[Volume\] section
        #[command(subcommand)]
        volume: Volume,
    },

    /// Generate a podman quadlet `.image` file
    ///
    /// For details on options see:
    /// https://docs.podman.io/en/stable/markdown/podman-pull.1.html
    Image {
        /// The \[Image\] section
        #[command(subcommand)]
        image: Box<Image>,
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
            PodmanCommands::Kube { kube } => (*kube).into(),
            PodmanCommands::Network { network } => (*network).into(),
            PodmanCommands::Volume { volume } => volume.into(),
            PodmanCommands::Image { image } => (*image).into(),
        }
    }
}

impl PodmanCommands {
    /// Convert the podman command into a [`quadlet::File`].
    fn into_quadlet(
        self,
        name: Option<String>,
        unit: Option<Unit>,
        globals: Globals,
        install: Option<quadlet::Install>,
    ) -> quadlet::File {
        let service = self.service().cloned();
        quadlet::File {
            name: name.unwrap_or_else(|| self.name().into()),
            unit,
            resource: self.into(),
            globals,
            service,
            install,
        }
    }

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
            Self::Image { image } => image.name(),
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

    /// Returns [`Some`] if a [`File::Quadlet`].
    fn quadlet_file(&self) -> Option<&quadlet::File> {
        match self {
            Self::Quadlet(file) => Some(file),
            Self::KubePod { .. } => None,
        }
    }

    /// Returns [`Some`] if a [`File::Quadlet`].
    fn quadlet_file_mut(&mut self) -> Option<&mut quadlet::File> {
        match self {
            Self::Quadlet(file) => Some(file),
            Self::KubePod { .. } => None,
        }
    }

    /// If a quadlet file, make all host paths absolute and clean.
    ///
    /// Relative paths are resolved using `resolve_dir` as the base.
    fn absolutize_host_paths(&mut self, resolve_dir: &Path) {
        for path in self.host_paths() {
            *path = absolute_clean_path(resolve_dir, path);
        }
    }

    fn write(&self, path: &FilePath, overwrite: bool) -> color_eyre::Result<()> {
        let path = path.to_full(self);
        let mut file = open_file(&path, overwrite)?;

        let path = path.display();
        write!(file, "{self}").wrap_err_with(|| format!("Failed to write to file: {path}"))?;
        println!("Wrote to file: {path}");

        Ok(())
    }
}

/// If `path` is relative, it is resolved using `resolve_dir` and a cleaned version is returned.
fn absolute_clean_path(resolve_dir: &Path, path: &Path) -> PathBuf {
    // Paths starting with "%" are also absolute because they start with a systemd specifier.
    let path: Cow<Path> = if path.is_absolute() || path.starts_with("%") {
        path.into()
    } else {
        resolve_dir.join(path).into()
    };

    path.clean()
}

fn open_file(path: impl AsRef<Path>, overwrite: bool) -> color_eyre::Result<fs::File> {
    fs::File::options()
        .write(true)
        .truncate(true)
        .create_new(!overwrite)
        .create(overwrite)
        .open(&path)
        .map_err(|error| {
            let path = path.as_ref().display();
            match error.kind() {
                io::ErrorKind::AlreadyExists => {
                    eyre::eyre!("File already exists, not overwriting it: {path}")
                        .suggestion("Use `--overwrite` if you wish overwrite existing files.")
                }
                _ => color_eyre::Report::new(error)
                    .wrap_err(format!("Failed to create/open file: {path}"))
                    .suggestion(
                        "Make sure the directory exists \
                                and you have write permissions for the file",
                    ),
            }
        })
}

impl HostPaths for File {
    fn host_paths(&mut self) -> impl Iterator<Item = &mut PathBuf> {
        self.quadlet_file_mut()
            .into_iter()
            .flat_map(quadlet::File::host_paths)
    }
}

impl Downgrade for File {
    fn downgrade(&mut self, version: PodmanVersion) -> Result<(), DowngradeError> {
        match self {
            Self::Quadlet(file) => file.downgrade(version),
            Self::KubePod { .. } => Ok(()),
        }
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
