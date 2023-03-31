use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Opt {
    Type(String),
    Device(String),
    Copy,
    Mount(Vec<Mount>),
}

impl FromStr for Opt {
    type Err = ParseOptError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            s if s.starts_with("type=") => {
                let (_, opt_type) = s.split_once('=').expect("delimiter is in guard");
                Ok(Self::Type(String::from(opt_type)))
            }
            s if s.starts_with("device=") => {
                let (_, device) = s.split_once('=').expect("delimiter is in guard");
                Ok(Self::Device(String::from(device)))
            }
            "copy" => Ok(Self::Copy),
            s if s.starts_with("o=") => {
                let (_, options) = s.split_once('=').expect("delimiter is in guard");
                let options = options
                    .split(',')
                    .map(str::parse)
                    .collect::<Result<Vec<_>, ()>>()
                    .expect("Mount::from_str cannot error");
                Ok(Self::Mount(options))
            }
            _ => Err(ParseOptError::InvalidVolumeDriverOption(String::from(s))),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseOptError {
    #[error("`{0}` is not a valid volume driver option")]
    InvalidVolumeDriverOption(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mount {
    Uid(String),
    Gid(String),
    Other(String),
}

impl FromStr for Mount {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("uid=") {
            let (_, uid) = s.split_once('=').expect("delimiter is in guard");
            Ok(Self::Uid(String::from(uid)))
        } else if s.starts_with("gid=") {
            let (_, gid) = s.split_once('=').expect("delimiter is in guard");
            Ok(Self::Gid(String::from(gid)))
        } else {
            Ok(Self::Other(String::from(s)))
        }
    }
}
