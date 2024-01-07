use std::{convert::Infallible, str::FromStr};

use thiserror::Error;

/// Options from `podman volume create --opt`
#[derive(Debug, Clone, PartialEq)]
pub enum Opt {
    /// `--opt type=`
    Type(String),
    /// `--opt device=`
    Device(String),
    /// `--opt copy`
    Copy,
    /// `--opt o=`
    Mount(Vec<Mount>),
    /// `--opt image=`
    Image(String),
}

impl Opt {
    /// Parse from an `option` and its `value`,
    /// equivalent to `podman volume create --opt <option>[=<value>]`.
    pub fn parse(option: &str, value: Option<String>) -> Result<Self, ParseOptError> {
        match (option, value) {
            ("type", Some(opt_type)) => Ok(Self::Type(opt_type)),
            ("device", Some(device)) => Ok(Self::Device(device)),
            ("copy", None) => Ok(Self::Copy),
            ("o", Some(options)) => Ok(Self::Mount(options.split(',').map(Mount::parse).collect())),
            ("image", Some(image)) => Ok(Self::Image(image)),
            (option, value) => Err(ParseOptError::InvalidVolumeDriverOption(
                value.map_or_else(|| option.into(), |value| format!("{option}={value}")),
            )),
        }
    }
}

impl From<Vec<Opt>> for crate::quadlet::Volume {
    fn from(value: Vec<Opt>) -> Self {
        value.into_iter().fold(Self::default(), |mut volume, opt| {
            match opt {
                Opt::Type(fs_type) => volume.fs_type = Some(fs_type),
                Opt::Device(device) => volume.device = Some(device),
                Opt::Copy => volume.copy = true,
                Opt::Mount(mount_opts) => {
                    for opt in mount_opts {
                        match opt {
                            Mount::Uid(uid) => volume.user = Some(uid),
                            Mount::Gid(gid) => volume.group = Some(gid),
                            Mount::Other(mount_opt) => {
                                if let Some(options) = volume.options.as_mut() {
                                    *options = format!("{options},{mount_opt}");
                                } else {
                                    volume.options = Some(mount_opt);
                                }
                            }
                        }
                    }
                }
                Opt::Image(image) => volume.image = Some(image),
            }
            volume
        })
    }
}

impl FromStr for Opt {
    type Err = ParseOptError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((option, value)) = s.split_once('=') {
            Self::parse(option, Some(value.into()))
        } else {
            Self::parse(s, None)
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseOptError {
    #[error("`{0}` is not a valid volume driver option")]
    InvalidVolumeDriverOption(String),
}

/// Mount options
#[derive(Debug, Clone, PartialEq)]
pub enum Mount {
    Uid(String),
    Gid(String),
    Other(String),
}

impl Mount {
    /// Parse from a string
    pub fn parse(s: &str) -> Self {
        if let Some(uid) = s.strip_prefix("uid=") {
            Self::Uid(uid.to_owned())
        } else if let Some(gid) = s.strip_prefix("gid=") {
            Self::Gid(gid.to_owned())
        } else {
            Self::Other(s.to_owned())
        }
    }
}

impl FromStr for Mount {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}
