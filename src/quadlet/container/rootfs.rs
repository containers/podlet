//! Provides [`Rootfs`] for the `rootfs` field on [`Container`](super::Container).

use std::{
    fmt::{self, Display, Formatter, Write},
    path::PathBuf,
    str::FromStr,
};

use serde::{Serialize, Serializer};
use thiserror::Error;

use super::mount::{idmap::ParseIdmapError, Idmap};

/// An exploded container on the file system.
///
/// The [`Display`] and [`FromStr`] implementations use the format
/// "PATH\[:\[O\]\[,idmap\[=IDMAP\]\]\]".
///
/// See the [`--rootfs`](https://docs.podman.io/en/stable/markdown/podman-run.1.html#rootfs) section
/// of **podman-run(1)**.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rootfs {
    /// Path to exploded container directory.
    pub path: PathBuf,

    /// Mount the `path` as storage using the overlay file system.
    pub overlay: bool,

    /// Create an idmapped mount to the target user namespace in the container.
    pub idmap: Option<Idmap>,
}

impl FromStr for Rootfs {
    type Err = ParseRootfsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Format is "PATH[:[O][,idmap[=IDMAP]]]".
        if let Some((path, options)) = s.rsplit_once(':') {
            let mut overlay = false;
            let mut idmap = None;

            for option in options.split_terminator(',') {
                if option == "O" {
                    overlay = true;
                } else if let Some(option) = option.strip_prefix("idmap") {
                    // idmap option format is "idmap[=IDMAP]".
                    let option = option
                        .strip_prefix('=')
                        .map(str::parse)
                        .transpose()?
                        .unwrap_or_default();
                    idmap = Some(option);
                } else {
                    return Err(ParseRootfsError::UnknownOption(option.into()));
                }
            }

            Ok(Self {
                path: path.into(),
                overlay,
                idmap,
            })
        } else {
            // path only, default options
            Ok(Self {
                path: s.into(),
                overlay: false,
                idmap: None,
            })
        }
    }
}

impl Display for Rootfs {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Self {
            path,
            overlay,
            idmap,
        } = self;

        // Format is "PATH[:[O][,idmap[=IDMAP]]]".

        path.display().fmt(f)?;

        if *overlay || idmap.is_some() {
            f.write_char(':')?;
        }

        if *overlay {
            f.write_char('O')?;
        }

        if let Some(idmap) = idmap {
            if *overlay {
                f.write_char(',')?;
            }

            f.write_str("idmap")?;

            if !idmap.is_empty() {
                f.write_char('=')?;
                Display::fmt(idmap, f)?;
            }
        }

        Ok(())
    }
}

impl Serialize for Rootfs {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

/// Error returned when parsing [`Rootfs`] from a string.
#[derive(Error, Debug)]
pub enum ParseRootfsError {
    /// Unknown rootfs option given.
    #[error("unknown rootfs option: {0}")]
    UnknownOption(String),

    /// Error while parsing [`Idmap`] option.
    #[error("error parsing idmap")]
    Idmap(#[from] ParseIdmapError),
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::super::mount::idmap::Mapping;

    use super::*;

    #[test]
    fn path() {
        let string = "/test";
        let rootfs: Rootfs = string.parse().unwrap();

        assert_eq!(
            rootfs,
            Rootfs {
                path: "/test".into(),
                overlay: false,
                idmap: None,
            },
        );
        assert_eq!(rootfs.to_string(), string);
    }

    #[test]
    fn overlay() {
        let string = "/test:O";
        let rootfs: Rootfs = string.parse().unwrap();

        assert_eq!(
            rootfs,
            Rootfs {
                path: "/test".into(),
                overlay: true,
                idmap: None,
            },
        );
        assert_eq!(rootfs.to_string(), string);
    }

    #[test]
    fn idmap() {
        let string = "/test:idmap=uids=0-1-2";
        let rootfs: Rootfs = string.parse().unwrap();

        assert_eq!(
            rootfs,
            Rootfs {
                path: "/test".into(),
                overlay: false,
                idmap: Some(Idmap {
                    uids: vec![Mapping {
                        container_relative: false,
                        from: 0,
                        to: 1,
                        length: 2,
                    }],
                    gids: Vec::new()
                }),
            },
        );
        assert_eq!(rootfs.to_string(), string);
    }

    #[test]
    fn overlay_and_idmap() {
        let string = "/test:O,idmap";
        let rootfs: Rootfs = string.parse().unwrap();

        assert_eq!(
            rootfs,
            Rootfs {
                path: "/test".into(),
                overlay: true,
                idmap: Some(Idmap::default()),
            },
        );
        assert_eq!(rootfs.to_string(), string);
    }
}
