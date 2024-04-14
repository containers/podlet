//! Provides [`Device`] for `AddDevice=` quadlet option of [`Container`](super::Container).

use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
    iter,
    path::PathBuf,
    str::FromStr,
};

use compose_spec::service::{self, device::Permissions};
use serde::{Serialize, Serializer};
use thiserror::Error;

use crate::quadlet::HostPaths;

/// Device to attach to a [`Container`](super::Container).
///
/// Format for the [`FromStr`] and [`Display`] implementations is
/// "host-device\[:container-device\]\[:permissions\]".
///
/// See `AddDevice=` under `[Container]` in
/// [**podman-systemd.unit(5)**](https://docs.podman.io/en/stable/markdown/podman-systemd.unit.5.html#adddevice).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    /// Host path.
    pub host: PathBuf,

    /// Container path.
    pub container: Option<PathBuf>,

    /// Read permission.
    pub read: bool,

    /// Write permission.
    pub write: bool,

    /// **mknod(2)** permission.
    pub mknod: bool,
}

impl HostPaths for Device {
    fn host_paths(&mut self) -> impl Iterator<Item = &mut PathBuf> {
        iter::once(&mut self.host)
    }
}

impl FromStr for Device {
    type Err = ParseDeviceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Format is "host-device[:container-device][:permissions]".
        // Supported permutations include: "host", "host:container", "host:container:",
        // "host:container:permissions", "host::permissions", and "host:permissions:permissions".

        let mut split = s.splitn(3, ':');

        let host = split.next().expect("split contains at least one element");
        let host = if host.is_empty() {
            return Err(ParseDeviceError::EmptyHostPath);
        } else {
            host.into()
        };

        // container or permissions
        let second = split.next().unwrap_or_default();

        // permissions
        let third = split.next().unwrap_or_default();

        let (container, permissions) = if second.starts_with('/') {
            // host:container[:permissions]
            let container = second.into();
            let permissions = Cow::Borrowed(third);
            (Some(container), permissions)
        } else {
            let permissions = match (second.is_empty(), third.is_empty()) {
                // host:permissions:permissions
                (false, false) => {
                    let mut permissions = String::from(second);
                    permissions.push_str(third);
                    permissions.into()
                }
                // host:permissions[:]
                (false, true) => second.into(),
                // host::permissions
                (true, false) => third.into(),
                // host only
                (true, true) => Cow::Borrowed(""),
            };
            (None, permissions)
        };

        let mut read = false;
        let mut write = false;
        let mut mknod = false;
        for char in permissions.chars() {
            match char {
                'r' => read = true,
                'w' => write = true,
                'm' => mknod = true,
                char => return Err(ParseDeviceError::UnknownPermission(char)),
            }
        }

        Ok(Self {
            host,
            container,
            read,
            write,
            mknod,
        })
    }
}

/// Error returned when parsing [`Device`].
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParseDeviceError {
    /// Host device path was empty
    #[error("host device path cannot be empty")]
    EmptyHostPath,

    /// An unknown permission was given.
    #[error("unknown permission '{0}'")]
    UnknownPermission(char),
}

impl Display for Device {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Self {
            host,
            container,
            read,
            write,
            mknod,
        } = self;

        // Format is "host-device[:container-device][:permissions]"

        host.display().fmt(f)?;

        if let Some(container) = container {
            f.write_str(":")?;
            container.display().fmt(f)?;
        }

        if *read || *write || *mknod {
            f.write_str(":")?;
        }

        if *read {
            f.write_str("r")?;
        }

        if *write {
            f.write_str("w")?;
        }

        if *mknod {
            f.write_str("m")?;
        }

        Ok(())
    }
}

impl Serialize for Device {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl From<service::Device> for Device {
    fn from(
        service::Device {
            host_path: host,
            container_path: container,
            permissions: Permissions { read, write, mknod },
        }: service::Device,
    ) -> Self {
        Self {
            host,
            container: Some(container),
            read,
            write,
            mknod,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn host() {
        let string = "/host";
        let device = Device::from_str(string).unwrap();

        assert_eq!(
            device,
            Device {
                host: string.into(),
                container: None,
                read: false,
                write: false,
                mknod: false,
            },
        );

        assert_eq!(device.to_string(), string);

        assert_eq!(device, "/host::".parse().unwrap());
    }

    #[test]
    fn host_container() {
        let string = "/host:/container";
        let device = Device::from_str(string).unwrap();

        assert_eq!(
            device,
            Device {
                host: "/host".into(),
                container: Some("/container".into()),
                read: false,
                write: false,
                mknod: false,
            },
        );

        assert_eq!(device.to_string(), string);

        assert_eq!(device, "/host:/container:".parse().unwrap());
    }

    #[test]
    fn host_permissions() {
        let string = "/host:rwm";
        let device = Device::from_str(string).unwrap();

        assert_eq!(
            device,
            Device {
                host: "/host".into(),
                container: None,
                read: true,
                write: true,
                mknod: true,
            },
        );

        assert_eq!(device.to_string(), string);

        assert_eq!(device, "/host::rwm".parse().unwrap());
        assert_eq!(device, "/host:r:wm".parse().unwrap());
    }

    #[test]
    fn host_container_permissions() {
        let string = "/host:/container:rwm";
        let device = Device::from_str(string).unwrap();

        assert_eq!(
            device,
            Device {
                host: "/host".into(),
                container: Some("/container".into()),
                read: true,
                write: true,
                mknod: true,
            },
        );

        assert_eq!(device.to_string(), string);
    }

    #[test]
    fn empty_host_err() {
        assert_eq!(
            Device::from_str(":/container").unwrap_err(),
            ParseDeviceError::EmptyHostPath,
        );
        assert_eq!(
            Device::from_str("").unwrap_err(),
            ParseDeviceError::EmptyHostPath,
        );
    }

    #[test]
    fn unknown_permission_err() {
        assert_eq!(
            Device::from_str("/host:a").unwrap_err(),
            ParseDeviceError::UnknownPermission('a'),
        );
    }
}
