//! Provides [`Volume`] for the `volume` field of [`Container`](super::Container).

use std::{
    convert::Infallible,
    fmt::{self, Display, Formatter, Write},
    path::PathBuf,
    str::FromStr,
};

use color_eyre::eyre::ensure;
use compose_spec::{
    Identifier,
    service::volumes::{
        self, HostPath, ShortOptions, ShortVolume,
        mount::{self, Bind, BindOptions, Common, VolumeOptions},
    },
};
use serde::{Serialize, Serializer};
use thiserror::Error;

use crate::quadlet::HostPaths;

use super::mount::{BindPropagation, Idmap, SELinuxRelabel, idmap::ParseIdmapError};

/// Volume to mount to a [`Container`](super::Container).
///
/// See the `--volume` section of
/// [**podman-run(1)**](https://docs.podman.io/en/stable/markdown/podman-run.1.html#volume-v-source-volume-host-dir-container-dir-options).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Volume {
    /// Optional source of the volume.
    pub source: Option<Source>,

    /// Path within the container to mount the volume to.
    pub container_path: PathBuf,

    /// Options for how to mount `source` into the container.
    pub options: Options,
}

impl HostPaths for Volume {
    fn host_paths(&mut self) -> impl Iterator<Item = &mut PathBuf> {
        self.source.host_paths().chain(self.options.host_paths())
    }
}

impl Volume {
    /// Create [`Volume`] from a container path.
    pub(crate) fn new(container_path: PathBuf) -> Self {
        Self {
            source: None,
            container_path,
            options: Options::default(),
        }
    }

    /// Parse [`Volume`] from its components.
    fn parse(
        source: Option<&str>,
        container_path: &str,
        options: Option<&str>,
    ) -> Result<Self, ParseVolumeError> {
        if container_path.starts_with('/') {
            let options = options.map(str::parse).transpose()?.unwrap_or_default();
            Ok(Self {
                source: source.map(Into::into),
                container_path: container_path.into(),
                options,
            })
        } else {
            Err(ParseVolumeError::ContainerPathNotAbsolute(
                container_path.to_owned(),
            ))
        }
    }
}

impl FromStr for Volume {
    type Err = ParseVolumeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Format is "[source:]container_path[:options]".
        let mut split = s.splitn(3, ':');
        let source_or_container = split.next().expect("split has at least one element");

        if let Some(container_path) = split.next() {
            Self::parse(Some(source_or_container), container_path, split.next())
        } else {
            Self::parse(None, source_or_container, None)
        }
    }
}

/// Error returned when parsing [`Volume`] from a string.
#[derive(Error, Debug)]
pub enum ParseVolumeError {
    /// Given container path was not an absolute path.
    #[error("container path `{0}` must be an absolute path")]
    ContainerPathNotAbsolute(String),

    /// Error while parsing [`Options`].
    #[error("error parsing volume option")]
    Options(#[from] ParseOptionsError),
}

impl Display for Volume {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Self {
            source,
            container_path,
            options,
        } = self;

        // Format is "[source:]container_path[:options]".

        if let Some(source) = source {
            source.fmt(f)?;
            f.write_char(':')?;
        }

        container_path.display().fmt(f)?;
        options.fmt(f)
    }
}

impl Serialize for Volume {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl From<ShortVolume> for Volume {
    fn from(
        ShortVolume {
            container_path,
            options,
        }: ShortVolume,
    ) -> Self {
        let container_path = container_path.into_inner();

        if let Some(ShortOptions {
            source,
            read_only,
            selinux,
        }) = options
        {
            Self {
                source: Some(source.into()),
                container_path,
                options: Options {
                    read_only,
                    selinux_relabel: selinux.map(Into::into),
                    ..Options::default()
                },
            }
        } else {
            Self::new(container_path)
        }
    }
}

impl TryFrom<mount::Volume> for Volume {
    type Error = color_eyre::Report;

    fn try_from(
        mount::Volume {
            source,
            volume,
            common:
                Common {
                    target,
                    read_only,
                    consistency,
                    extensions,
                },
        }: mount::Volume,
    ) -> Result<Self, Self::Error> {
        ensure!(
            consistency.is_none(),
            "`consistency` volume option is not supported"
        );
        ensure!(
            extensions.is_empty(),
            "compose extensions are not supported"
        );

        let mut options = Options::try_from(volume.unwrap_or_default())?;
        options.read_only = read_only;

        Ok(Self {
            source: source.map(Into::into),
            container_path: target.into_inner(),
            options,
        })
    }
}

impl TryFrom<Bind> for Volume {
    type Error = color_eyre::Report;

    fn try_from(
        Bind {
            source,
            bind,
            common:
                Common {
                    target,
                    read_only,
                    consistency,
                    extensions,
                },
        }: Bind,
    ) -> Result<Self, Self::Error> {
        ensure!(
            consistency.is_none(),
            "`consistency` volume option is not supported"
        );
        ensure!(
            extensions.is_empty(),
            "compose extensions are not supported"
        );

        let mut options = Options::try_from(bind.unwrap_or_default())?;
        options.read_only = read_only;

        Ok(Self {
            source: Some(source.into()),
            container_path: target.into_inner(),
            options,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Source of a [`Volume`].
pub enum Source {
    /// Named volume source.
    NamedVolume(String),

    /// Host path source.
    HostPath(PathBuf),
}

impl HostPaths for Source {
    fn host_paths(&mut self) -> impl Iterator<Item = &mut PathBuf> {
        match self {
            Self::NamedVolume(_) => None,
            Self::HostPath(path) => Some(path),
        }
        .into_iter()
    }
}

impl Source {
    /// Parse [`Source`] from a stringy type.
    fn parse<T>(source: T) -> Self
    where
        T: AsRef<str> + Into<String> + Into<PathBuf>,
    {
        if source.as_ref().starts_with(['.', '/', '~', '%']) {
            Self::HostPath(source.into())
        } else {
            Self::NamedVolume(source.into())
        }
    }
}

impl From<&str> for Source {
    fn from(value: &str) -> Self {
        Self::parse(value)
    }
}

impl From<String> for Source {
    fn from(value: String) -> Self {
        Self::parse(value)
    }
}

impl FromStr for Source {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.into())
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::NamedVolume(volume) => f.write_str(volume),
            Self::HostPath(path) => path.display().fmt(f),
        }
    }
}

impl From<volumes::Source> for Source {
    fn from(value: volumes::Source) -> Self {
        match value {
            volumes::Source::HostPath(host_path) => Self::HostPath(host_path.into_inner()),
            volumes::Source::Volume(volume) => Self::NamedVolume(volume.into()),
        }
    }
}

impl From<Identifier> for Source {
    fn from(value: Identifier) -> Self {
        Self::NamedVolume(value.into())
    }
}

impl From<HostPath> for Source {
    fn from(value: HostPath) -> Self {
        Self::HostPath(value.into_inner())
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// Mount volume in read only mode.
    pub read_only: bool,

    /// Relabel objects within the volume.
    pub selinux_relabel: Option<SELinuxRelabel>,

    /// Mount volume using the overlay file system.
    pub overlay: Option<Overlay>,

    /// Recursively change the host UID and GID of each file in the volume to match the container's
    /// user namespace.
    pub chown: bool,

    /// Do not copy contents from destination directory onto newly created volumes.
    pub no_copy: bool,

    /// Whether devices in the volume can be used in the container.
    pub devices: bool,

    /// Do not allow executables in the volume to be executed within the container.
    pub no_executables: bool,

    /// Allow SUID executables in the volume to be used within the container to change privileges.
    pub suid: bool,

    /// Recursively mount a volume and all of its submounts into the container.
    pub recursive_bind: bool,

    /// Set how mounts in the volume in the container are propagated to the host and vice versa.
    pub bind_propagation: BindPropagation,

    /// Create an idmapped mount to the target user namespace in the container.
    pub idmap: Option<Idmap>,
}

impl HostPaths for Options {
    fn host_paths(&mut self) -> impl Iterator<Item = &mut PathBuf> {
        self.overlay.host_paths()
    }
}

impl Options {
    /// Parse `option` and fold it into `self`.
    fn try_fold(mut self, option: &str) -> Result<Self, ParseOptionsError> {
        // option could be "option" or "option=value".
        let (option, value) = option
            .split_once('=')
            .map_or((option, None), |(option, value)| {
                (option, (!value.is_empty()).then_some(value))
            });

        match option {
            "rw" => self.read_only = false,
            "ro" => self.read_only = true,
            "z" => self.selinux_relabel = Some(SELinuxRelabel::Shared),
            "Z" => self.selinux_relabel = Some(SELinuxRelabel::Private),
            "O" => set_option("O", &mut self.overlay, Overlay::default)?,
            "upperdir" => {
                let value = value.ok_or(ParseOptionsError::RequiresValue("upperdir"))?;
                let overlay = self
                    .overlay
                    .as_mut()
                    .ok_or(ParseOptionsError::OverlayMissing)?;
                set_option("upperdir", &mut overlay.upper_dir, || value.into())?;
            }
            "workdir" => {
                let value = value.ok_or(ParseOptionsError::RequiresValue("workdir"))?;
                let overlay = self
                    .overlay
                    .as_mut()
                    .ok_or(ParseOptionsError::OverlayMissing)?;
                set_option("workdir", &mut overlay.work_dir, || value.into())?;
            }
            "U" => self.chown = true,
            "nocopy" => self.no_copy = true,
            "copy" => self.no_copy = false,
            "nodev" => self.devices = false,
            "dev" => self.devices = true,
            "noexec" => self.no_executables = true,
            "exec" => self.no_executables = false,
            "nosuid" => self.suid = false,
            "suid" => self.suid = true,
            "rbind" => self.recursive_bind = true,
            "bind" => self.recursive_bind = false,
            "idmap" => {
                if self.idmap.is_some() {
                    return Err(ParseOptionsError::Multiple("idmap"));
                }
                let value = value.map(str::parse).transpose()?.unwrap_or_default();
                self.idmap = Some(value);
            }
            option => {
                self.bind_propagation = option
                    .parse()
                    .map_err(|_| ParseOptionsError::Unknown(option.to_owned()))?;
            }
        }

        Ok(self)
    }
}

/// Set `option` with `value` if it isn't already occupied.
fn set_option<T>(
    name: &'static str,
    option: &mut Option<T>,
    value: impl FnOnce() -> T,
) -> Result<(), ParseOptionsError> {
    if option.is_some() {
        Err(ParseOptionsError::Multiple(name))
    } else {
        *option = Some(value());
        Ok(())
    }
}

impl FromStr for Options {
    type Err = ParseOptionsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Format is ":option[=value],...".
        s.strip_prefix(':')
            .unwrap_or(s)
            .split_terminator(',')
            .try_fold(Self::default(), Self::try_fold)
    }
}

/// Error returned when parsing [`Options`] from a string.
#[derive(Error, Debug)]
pub enum ParseOptionsError {
    /// Multiples of an option given.
    #[error("multiple `{0}` options given")]
    Multiple(&'static str),

    /// Option that requires a value was not given one.
    #[error("option `{0}` requires a value")]
    RequiresValue(&'static str),

    /// Overlay not set before `upperdir` or `workdir`.
    #[error("`upperdir` and `workdir` options require that `O` is specified first")]
    OverlayMissing,

    /// Error parsing [`Idmap`].
    #[error("error parsing idmap")]
    Idmap(#[from] ParseIdmapError),

    /// Unknown volume option given.
    #[error("unknown volume option: {0}")]
    Unknown(String),
}

impl Display for Options {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut f = OptionsFormatter::new(f);

        let Self {
            read_only,
            selinux_relabel,
            ref overlay,
            chown,
            no_copy,
            devices,
            no_executables,
            suid,
            recursive_bind,
            bind_propagation,
            ref idmap,
        } = *self;

        // Format is ":option,...".

        if read_only {
            f.write_option("ro")?;
        }

        if let Some(relabel) = selinux_relabel {
            f.write_option(char::from(relabel))?;
        }

        if let Some(overlay) = overlay {
            f.write_option(overlay)?;
        }

        if chown {
            f.write_option('U')?;
        }

        if no_copy {
            f.write_option("nocopy")?;
        }

        if devices {
            f.write_option("dev")?;
        }

        if no_executables {
            f.write_option("noexec")?;
        }

        if suid {
            f.write_option("suid")?;
        }

        if recursive_bind {
            f.write_option("rbind")?;
        }

        if bind_propagation != BindPropagation::default() {
            f.write_option(bind_propagation)?;
        }

        if let Some(idmap) = idmap {
            if idmap.is_empty() {
                f.write_option("idmap")?;
            } else {
                f.write_option(format_args!("idmap={idmap}"))?;
            }
        }

        Ok(())
    }
}

/// [`Formatter`] wrapper for [`Display`] implementation of [`Options`].
struct OptionsFormatter<'a, 'b> {
    formatter: &'a mut Formatter<'b>,
    first: bool,
}

impl<'a, 'b> OptionsFormatter<'a, 'b> {
    /// Create a new [`OptionsFormatter`] from a [`Formatter`].
    fn new(formatter: &'a mut Formatter<'b>) -> Self {
        Self {
            formatter,
            first: true,
        }
    }

    /// Write the first character (':' or ',') for writing an option.
    fn write_option(&mut self, option: impl Display) -> fmt::Result {
        if self.first {
            self.first = false;
            self.formatter.write_char(':')?;
        } else {
            self.formatter.write_char(',')?;
        }
        option.fmt(self.formatter)
    }
}

impl TryFrom<VolumeOptions> for Options {
    type Error = color_eyre::Report;

    fn try_from(
        VolumeOptions {
            nocopy: no_copy,
            subpath,
            extensions,
        }: VolumeOptions,
    ) -> Result<Self, Self::Error> {
        ensure!(
            subpath.is_none(),
            "`subpath` volume option is not supported"
        );
        ensure!(
            extensions.is_empty(),
            "compose extensions are not supported"
        );

        Ok(Self {
            no_copy,
            ..Self::default()
        })
    }
}

impl TryFrom<BindOptions> for Options {
    type Error = color_eyre::Report;

    fn try_from(
        BindOptions {
            propagation,
            create_host_path,
            selinux,
            extensions,
        }: BindOptions,
    ) -> Result<Self, Self::Error> {
        ensure!(
            !create_host_path,
            "`create_host_path` bind mount option is not supported"
        );
        ensure!(
            extensions.is_empty(),
            "compose extensions are not supported"
        );

        Ok(Self {
            bind_propagation: propagation.map_or_else(BindPropagation::default, Into::into),
            selinux_relabel: selinux.map(Into::into),
            ..Self::default()
        })
    }
}

/// Overlay mount options for [`Volume`] [`Options`].
///
/// See **fuse-overlayfs(1)**.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Overlay {
    /// A directory merged on top of all the lower directories where all the changes done to the
    /// file system will be written.
    pub upper_dir: Option<PathBuf>,

    /// A directory used internally by fuse-overlayfs.
    ///
    /// Must be on the same file system as the upper dir.
    pub work_dir: Option<PathBuf>,
}

impl HostPaths for Overlay {
    fn host_paths(&mut self) -> impl Iterator<Item = &mut PathBuf> {
        self.upper_dir.iter_mut().chain(&mut self.work_dir)
    }
}

impl Display for Overlay {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Self {
            upper_dir,
            work_dir,
        } = self;

        f.write_char('O')?;

        if let Some(upper_dir) = upper_dir {
            write!(f, ",upperdir={}", upper_dir.display())?;
        }

        if let Some(work_dir) = work_dir {
            write!(f, ",workdir={}", work_dir.display())?;
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn container_only() {
        let string = "/container/path";
        let volume: Volume = string.parse().unwrap();
        assert_eq!(volume, Volume::new("/container/path".into()));
        assert_eq!(volume.to_string(), string);
    }

    #[test]
    fn source_and_container() {
        let string = "/host/path:/container/path";
        let volume: Volume = string.parse().unwrap();
        assert_eq!(
            volume,
            Volume {
                source: Some(Source::HostPath("/host/path".into())),
                container_path: "/container/path".into(),
                options: Options::default(),
            },
        );
        assert_eq!(volume.to_string(), string);
    }

    #[test]
    fn all_options() {
        let string = "/host/path:/container/path:ro,Z,O,upperdir=/upper/dir,workdir=/work/dir,U,\
            nocopy,dev,noexec,suid,rbind,shared,idmap";
        let volume: Volume = string.parse().unwrap();
        let options = Options {
            read_only: true,
            selinux_relabel: Some(SELinuxRelabel::Private),
            overlay: Some(Overlay {
                upper_dir: Some("/upper/dir".into()),
                work_dir: Some("/work/dir".into()),
            }),
            chown: true,
            no_copy: true,
            devices: true,
            no_executables: true,
            suid: true,
            recursive_bind: true,
            bind_propagation: BindPropagation::Shared,
            idmap: Some(Idmap::default()),
        };
        assert_eq!(
            volume,
            Volume {
                source: Some(Source::HostPath("/host/path".into())),
                container_path: "/container/path".into(),
                options,
            }
        );
        assert_eq!(volume.to_string(), string);
    }
}
