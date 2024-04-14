//! Provides [`Mount`] for the `mount` field of [`Container`](super::Container).

pub(super) mod idmap;
mod mode;
mod tmpfs;

use std::{
    fmt::{self, Display, Formatter},
    ops::Not,
    path::PathBuf,
    str::FromStr,
};

use compose_spec::service::volumes::{mount, SELinux};
use serde::{
    de::{
        self,
        value::{MapAccessDeserializer, StrDeserializer},
        MapAccess, Visitor,
    },
    Deserialize, Deserializer, Serialize,
};
use thiserror::Error;
use umask::Mode;

use crate::{
    quadlet::HostPaths,
    serde::{mount_options, skip_default},
};

pub use self::{idmap::Idmap, tmpfs::Tmpfs};

/// Filesystem mount types to attach to a [`Container`](super::Container).
///
/// See the `Mount=` quadlet option in the `[Container]` sections of
/// [**podman-systemd.unit(5)**](https://docs.podman.io/en/stable/markdown/podman-systemd.unit.5.html#mount)
/// and `podman run --mount` in
/// [**podman-run(1)**](https://docs.podman.io/en/stable/markdown/podman-run.1.html#mount-type-type-type-specific-option).
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Mount {
    Bind(Bind),
    DevPts(DevPts),
    Glob(Bind),
    Image(Image),
    Ramfs(Tmpfs),
    Tmpfs(Tmpfs),
    Volume(Volume),
}

/// [`Mount`] variants, for deserializing "type" value.
#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum MountType {
    Bind,
    DevPts,
    Glob,
    Image,
    Ramfs,
    Tmpfs,
    Volume,
}

/// Match [`Mount`] variants to types to deserialize.
macro_rules! match_type_deserialize {
    ($value:expr, $map:expr, $($variant:ident => $kind:ty,)*) => {
        match $value {
            $(
                Self::$variant => Ok(Mount::$variant(<$kind>::deserialize($map)?)),
            )*
        }
    };
}

impl MountType {
    /// Deserialize [`Mount`] variant from a [`MapAccess`] based on the [`MountType`].
    fn into_mount<'de, A: MapAccess<'de>>(self, map: A) -> Result<Mount, A::Error> {
        let map = MapAccessDeserializer::new(map);
        match_type_deserialize! {
            self, map,
            Bind => Bind,
            DevPts => DevPts,
            Glob => Bind,
            Image => Image,
            Ramfs => Tmpfs,
            Tmpfs => Tmpfs,
            Volume => Volume,
        }
    }
}

// Not using #[derive(Deserialize)] because internally tagged enums are deserialized with
// `Deserializer::deserialize_any()` which doesn't work for `Option<Idmap>`.
impl<'de> Deserialize<'de> for Mount {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(MountVisitor)
    }
}

/// Deserialization [`Visitor`] for [`Mount`].
struct MountVisitor;

impl<'de> Visitor<'de> for MountVisitor {
    type Value = Mount;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("a map of mount options")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let (key, value) = map
            .next_entry::<&str, MountType>()?
            .ok_or_else(|| de::Error::missing_field("type"))?;
        if key == "type" {
            value.into_mount(map)
        } else {
            Err(de::Error::custom("\"type\" must be the first mount option"))
        }
    }
}

impl Display for Mount {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mount = mount_options::to_string(self).map_err(|_| fmt::Error)?;
        f.write_str(&mount)
    }
}

impl FromStr for Mount {
    type Err = ParseMountError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        mount_options::from_str(s).map_err(Into::into)
    }
}

/// Error returned when parsing [`Mount`] from a string.
// Used to give a better error message when parsing `podman run --mount`.
#[derive(Error, Debug)]
#[error("error while deserializing mount options: {0}")]
pub struct ParseMountError(String);

impl From<mount_options::Error> for ParseMountError {
    #[allow(clippy::panic)]
    fn from(value: mount_options::Error) -> Self {
        match value {
            mount_options::Error::Custom(error) => Self(error),
            mount_options::Error::BadType => panic!(
                "attempted to deserialize a type incompatible with the mount options deserializer"
            ),
        }
    }
}

impl HostPaths for Mount {
    fn host_paths(&mut self) -> impl Iterator<Item = &mut PathBuf> {
        match self {
            Self::Bind(bind) | Self::Glob(bind) => Some(&mut bind.source),
            Self::DevPts(_)
            | Self::Image(_)
            | Self::Ramfs(_)
            | Self::Tmpfs(_)
            | Self::Volume(_) => None,
        }
        .into_iter()
    }
}

/// Bind or glob type [`Mount`].
#[allow(clippy::struct_field_names)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Bind {
    /// Mount source spec.
    #[serde(alias = "src")]
    pub source: PathBuf,

    /// Mount destination spec.
    #[serde(
        default,
        alias = "dst",
        alias = "target",
        skip_serializing_if = "Option::is_none"
    )]
    pub destination: Option<PathBuf>,

    /// Only read permissions
    #[serde(
        default,
        rename = "readonly",
        alias = "ro",
        skip_serializing_if = "Not::not"
    )]
    pub read_only: bool,

    /// Bind propagation type.
    #[serde(default, skip_serializing_if = "skip_default")]
    pub bind_propagation: BindPropagation,

    /// Do not set up a recursive bind mount.
    #[serde(default, skip_serializing_if = "Not::not")]
    pub bind_nonrecursive: bool,

    /// SELinux relabeling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relabel: Option<SELinuxRelabel>,

    /// Create an idmapped mount to the target user namespace in the container.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idmap: Option<Idmap>,

    /// Change recursively the owner and group of the source volume based on the UID and GID of the
    /// container.
    #[serde(default, alias = "U", skip_serializing_if = "Not::not")]
    pub chown: bool,
}

impl Bind {
    /// Create a [`Bind`] from a source with defaults.
    #[cfg(test)]
    fn new(source: PathBuf) -> Self {
        Self {
            source,
            destination: None,
            read_only: false,
            bind_propagation: BindPropagation::default(),
            bind_nonrecursive: false,
            relabel: None,
            idmap: None,
            chown: false,
        }
    }
}

/// Types of bind propagation.
#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BindPropagation {
    Shared,
    Slave,
    Private,
    Unbindable,
    RShared,
    RSlave,
    RUnbindable,
    #[default]
    RPrivate,
}

impl FromStr for BindPropagation {
    type Err = ParseBindPropagationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::deserialize(StrDeserializer::<de::value::Error>::new(s))
            .map_err(|_| ParseBindPropagationError(s.to_owned()))
    }
}

#[derive(Error, Debug)]
#[error("unknown bind propagation type: {0}")]
pub struct ParseBindPropagationError(String);

impl Display for BindPropagation {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.serialize(f)
    }
}

impl From<mount::BindPropagation> for BindPropagation {
    fn from(value: mount::BindPropagation) -> Self {
        match value {
            mount::BindPropagation::Private => Self::Private,
            mount::BindPropagation::Shared => Self::Shared,
            mount::BindPropagation::Slave => Self::Slave,
            mount::BindPropagation::Unbindable => Self::Unbindable,
            mount::BindPropagation::RPrivate => Self::RPrivate,
            mount::BindPropagation::RShared => Self::RShared,
            mount::BindPropagation::RSlave => Self::RSlave,
            mount::BindPropagation::RUnbindable => Self::RUnbindable,
        }
    }
}

/// SELinux relabeling.
#[allow(clippy::doc_markdown)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SELinuxRelabel {
    Shared,
    Private,
}

impl From<SELinuxRelabel> for char {
    fn from(value: SELinuxRelabel) -> Self {
        match value {
            SELinuxRelabel::Shared => 'z',
            SELinuxRelabel::Private => 'Z',
        }
    }
}

impl From<SELinux> for SELinuxRelabel {
    fn from(value: SELinux) -> Self {
        match value {
            SELinux::Shared => Self::Shared,
            SELinux::Private => Self::Private,
        }
    }
}

/// devpts type [`Mount`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct DevPts {
    /// Mount destination spec.
    #[serde(alias = "dst", alias = "target")]
    pub destination: PathBuf,

    /// UID of the file owner.
    #[serde(default, skip_serializing_if = "skip_default")]
    pub uid: u32,

    /// GID of the file owner.
    #[serde(default, skip_serializing_if = "skip_default")]
    pub gid: u32,

    /// Permission mask for the file (default 600).
    #[serde(
        default = "mode::default",
        with = "mode",
        skip_serializing_if = "mode::skip_default"
    )]
    pub mode: Mode,

    /// Maximum number of PTYs (default 1048576).
    #[serde(default = "ptys::default", skip_serializing_if = "ptys::skip_default")]
    pub max: u32,
}

mod ptys {
    pub const fn default() -> u32 {
        1_048_576
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub const fn skip_default(ptys: &u32) -> bool {
        *ptys == default()
    }
}

impl DevPts {
    /// Create a [`DevPts`] from a destination with defaults.
    #[cfg(test)]
    fn new(destination: PathBuf) -> Self {
        Self {
            destination,
            uid: 0,
            gid: 0,
            mode: mode::default(),
            max: ptys::default(),
        }
    }
}

/// Image type [`Mount`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Image {
    /// Mount source spec.
    #[serde(alias = "src")]
    pub source: String,

    /// Mount destination spec.
    #[serde(alias = "dst", alias = "target")]
    pub destination: PathBuf,

    /// Read-write permission.
    #[serde(
        default,
        rename = "readwrite",
        alias = "rw",
        skip_serializing_if = "Not::not"
    )]
    pub read_write: bool,
}

/// Volume type [`Mount`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Volume {
    /// Mount source spec.
    #[serde(default, alias = "src", skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Mount destination spec.
    #[serde(alias = "dst", alias = "target")]
    pub destination: PathBuf,

    /// Only read permissions
    #[serde(
        default,
        rename = "readonly",
        alias = "ro",
        skip_serializing_if = "Not::not"
    )]
    pub read_only: bool,

    /// Change recursively the owner and group of the source volume based on the UID and GID of the
    /// container.
    #[serde(default, alias = "U", skip_serializing_if = "Not::not")]
    pub chown: bool,

    /// Create an idmapped mount to the target user namespace in the container.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idmap: Option<Idmap>,
}

impl Volume {
    /// Create a new [`Volume`] from a `destination`.
    #[cfg(test)]
    fn new(destination: PathBuf) -> Self {
        Self {
            source: None,
            destination,
            read_only: false,
            chown: false,
            idmap: None,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn bind() {
        let string = "type=bind,source=/src,destination=/dst";
        let mount: Mount = string.parse().unwrap();
        assert_eq!(
            mount,
            Mount::Bind(Bind {
                destination: Some("/dst".into()),
                ..Bind::new("/src".into())
            }),
        );
        assert_eq!(mount.to_string(), string);

        let string =
            "type=bind,source=/src,destination=/dst,readonly=true,bind-propagation=shared,\
                bind-nonrecursive=true,relabel=shared,idmap,chown=true";
        let mount: Mount = string.parse().unwrap();
        assert_eq!(
            mount,
            Mount::Bind(Bind {
                source: "/src".into(),
                destination: Some("/dst".into()),
                read_only: true,
                bind_propagation: BindPropagation::Shared,
                bind_nonrecursive: true,
                relabel: Some(SELinuxRelabel::Shared),
                idmap: Some(Idmap::default()),
                chown: true,
            }),
        );
        assert_eq!(mount.to_string(), string);
    }

    #[test]
    fn devpts() {
        let string = "type=devpts,destination=/dst";
        let mount: Mount = string.parse().unwrap();
        assert_eq!(mount, Mount::DevPts(DevPts::new("/dst".into())));
        assert_eq!(mount.to_string(), string);

        let string = "type=devpts,destination=/dst,uid=100,gid=100,mode=755,max=10";
        let mount: Mount = string.parse().unwrap();
        assert_eq!(
            mount,
            Mount::DevPts(DevPts {
                destination: "/dst".into(),
                uid: 100,
                gid: 100,
                mode: Mode::from(0o755),
                max: 10
            })
        );
        assert_eq!(mount.to_string(), string);
    }

    #[test]
    fn glob() {
        let string = "type=glob,source=/usr/lib/libfoo*,destination=/usr/lib,readonly=true";
        let mount: Mount = string.parse().unwrap();
        assert_eq!(
            mount,
            Mount::Glob(Bind {
                destination: Some("/usr/lib".into()),
                read_only: true,
                ..Bind::new("/usr/lib/libfoo*".into())
            })
        );
        assert_eq!(mount.to_string(), string);
    }

    #[test]
    fn image() {
        let string = "type=image,source=fedora,destination=/fedora-image,readwrite=true";
        let mount: Mount = string.parse().unwrap();
        assert_eq!(
            mount,
            Mount::Image(Image {
                source: "fedora".into(),
                destination: "/fedora-image".into(),
                read_write: true
            }),
        );
        assert_eq!(mount.to_string(), string);
    }

    #[test]
    fn ramfs() {
        let string = "type=ramfs,destination=/dst,readonly=true,\
                        tmpfs-size=256m,tmpfs-mode=755,notmpcopyup,chown=true";
        let mount: Mount = string.parse().unwrap();
        assert_eq!(
            mount,
            Mount::Ramfs(Tmpfs {
                destination: "/dst".into(),
                read_only: true,
                size: tmpfs::Size::Mebibytes(256),
                mode: Mode::from(0o755),
                tmpcopyup: false,
                chown: true
            }),
        );
        assert_eq!(mount.to_string(), string);
    }

    #[test]
    fn tmpfs() {
        let string = "type=tmpfs,destination=/dst,readonly=true,\
                        tmpfs-size=256m,tmpfs-mode=755,notmpcopyup,chown=true";
        let mount: Mount = string.parse().unwrap();
        assert_eq!(
            mount,
            Mount::Tmpfs(Tmpfs {
                destination: "/dst".into(),
                read_only: true,
                size: tmpfs::Size::Mebibytes(256),
                mode: Mode::from(0o755),
                tmpcopyup: false,
                chown: true
            }),
        );
        assert_eq!(mount.to_string(), string);
    }

    #[test]
    fn volume() {
        let string = "type=volume,destination=/dst";
        let mount: Mount = string.parse().unwrap();
        assert_eq!(mount, Mount::Volume(Volume::new("/dst".into())),);
        assert_eq!(mount.to_string(), string);

        let string =
            "type=volume,source=volume,destination=/dst,readonly=true,chown=true,idmap=uids=@0-1-2";
        let mount: Mount = string.parse().unwrap();
        assert_eq!(
            mount,
            Mount::Volume(Volume {
                source: Some("volume".into()),
                destination: "/dst".into(),
                read_only: true,
                chown: true,
                idmap: Some(Idmap {
                    uids: vec![idmap::Mapping {
                        container_relative: true,
                        from: 0,
                        to: 1,
                        length: 2,
                    }],
                    gids: Vec::new(),
                }),
            }),
        );
        assert_eq!(mount.to_string(), string);
    }
}
