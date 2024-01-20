//! Provides [`Idmap`] for the `idmap` fields of [`Bind`](super::Bind) and [`Volume`](super::Volume).

use std::{
    fmt::{self, Display, Formatter},
    num::ParseIntError,
    str::FromStr,
};

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

/// Idmap [`Bind`](super::Bind) and [`Volume`](super::Volume) [`Mount`](super::Mount) options.
///
/// (De)serializes from/to "uids=\[@\]from-to-length\[#...\];gids=\[@\]from-to-length\[#...\]"
/// or unit [`()`] if empty.
///
/// See the [`--mount`](https://docs.podman.io/en/stable/markdown/podman-run.1.html#mount-type-type-type-specific-option)
/// or [`--volume` "Idmapped mount"](https://docs.podman.io/en/stable/markdown/podman-run.1.html#volume-v-source-volume-host-dir-container-dir-options)
/// sections of **podman-run(1)**.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Idmap {
    pub uids: Vec<Mapping>,
    pub gids: Vec<Mapping>,
}

impl Idmap {
    /// Returns `true` if all fields are empty.
    pub fn is_empty(&self) -> bool {
        let Self { uids, gids } = self;
        uids.is_empty() && gids.is_empty()
    }
}

impl FromStr for Idmap {
    type Err = ParseIdmapError;

    #[allow(clippy::similar_names)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Format is "uids=0-1-10#10-11-10;gids=0-100-10".
        let (first, second) = s
            .split_once(';')
            .map_or((s, None), |(first, second)| (first, Some(second)));

        let (first_prefix, first) = split_prefix(first)?;
        let second = second.map(split_prefix).transpose()?;

        let (uids, gids) = match (first_prefix, first, second) {
            // uids only
            ("uids", uids, None) => (Some(uids), None),
            // gids only
            ("gids", gids, None) => (None, Some(gids)),
            // uids and gids
            ("uids", uids, Some(("gids", gids))) | ("gids", gids, Some(("uids", uids))) => {
                (Some(uids), Some(gids))
            }
            // uids repeated
            ("uids", _, Some(("uids", _))) => return Err(ParseIdmapError::RepeatedUidsPrefix),
            // gids repeated
            ("gids", _, Some(("gids", _))) => return Err(ParseIdmapError::RepeatedGidsPrefix),
            // neither uids or gids
            (prefix, _, None) | (_, _, Some((prefix, _))) => {
                return Err(ParseIdmapError::UnknownPrefix(prefix.into()));
            }
        };

        Ok(Self {
            uids: parse_mappings(uids)?,
            gids: parse_mappings(gids)?,
        })
    }
}

/// Split `s` on '=', [`ParseIdmapError::MissingPrefix`] otherwise.
fn split_prefix(s: &str) -> Result<(&str, &str), ParseIdmapError> {
    s.split_once('=').ok_or(ParseIdmapError::MissingPrefix)
}

/// Parse [`Mapping`]s from a string.
///
/// Mappings are interspersed with "#".
fn parse_mappings(ids: Option<&str>) -> Result<Vec<Mapping>, ParseMappingError> {
    ids.map_or_else(
        || Ok(Vec::new()),
        |s| s.split('#').map(Mapping::from_str).collect(),
    )
}

/// Error returned when parsing [`Idmap`].
#[derive(Error, Debug)]
pub enum ParseIdmapError {
    /// A mapping is missing a "uids=" or "gids=" prefix.
    #[error("mapping must be prefixed with \"uids=\" or \"gids=\"")]
    MissingPrefix,

    /// "uids=" set twice.
    #[error("\"uids=\" prefix repeated")]
    RepeatedUidsPrefix,

    /// "gids=" set twice.
    #[error("\"gids=\" prefix repeated")]
    RepeatedGidsPrefix,

    /// Unknown prefix given.
    #[error("unknown mapping prefix: {0}=")]
    UnknownPrefix(String),

    /// Error parsing [`Mapping`].
    #[error("error parsing UID/GID mapping")]
    ParseMapping(#[from] ParseMappingError),
}

impl Display for Idmap {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Self { uids, gids } = self;

        // Format is "uids=0-1-10#10-11-10;gids=0-100-10".

        if !uids.is_empty() {
            f.write_str("uids=")?;
            fmt_mappings(uids, f)?;
        }

        if !gids.is_empty() {
            if !uids.is_empty() {
                f.write_str(";")?;
            }

            f.write_str("gids=")?;
            fmt_mappings(gids, f)?;
        }

        Ok(())
    }
}

/// Write a slice of [`Mapping`]s to `f`.
///
/// Mappings are interspersed with "#".
fn fmt_mappings(slice: &[Mapping], f: &mut Formatter) -> fmt::Result {
    let mut iter = slice.iter();
    if let Some(first) = iter.next() {
        Display::fmt(first, f)?;
    }
    for mapping in iter {
        f.write_str("#")?;
        Display::fmt(mapping, f)?;
    }
    Ok(())
}

impl Serialize for Idmap {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.is_empty() {
            serializer.serialize_unit()
        } else {
            serializer.collect_str(self)
        }
    }
}

impl<'de> Deserialize<'de> for Idmap {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(Visitor)
    }
}

struct Visitor;

impl<'de> de::Visitor<'de> for Visitor {
    type Value = Idmap;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("unit or a string")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        v.parse().map_err(de::Error::custom)
    }

    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(Idmap::default())
    }
}

/// Custom UID/GID mapping for [`Idmap`].
///
/// [`Display`] and [`FromStr`] format is "[@]from-to-length".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mapping {
    /// Whether `from` and `to` are relative to the container user namespace.
    pub container_relative: bool,
    /// Start of the backing file system IDs.
    pub from: u32,
    /// Start of the mapped IDs.
    pub to: u32,
    /// Length of the mapping.
    pub length: u32,
}

impl FromStr for Mapping {
    type Err = ParseMappingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Format is "[@]from-to-length".
        let mut container_relative = false;
        let s = s.strip_prefix('@').map_or(s, |s| {
            container_relative = true;
            s
        });

        let mut split = s.split('-');

        let count = split.clone().count();
        if count != 3 {
            return Err(ParseMappingError::NotTriplet { count });
        }

        let mut next = || {
            let value = split.next().unwrap_or_default();
            value.parse().map_err(|source| ParseMappingError::ParseInt {
                source,
                value: value.into(),
            })
        };

        Ok(Self {
            container_relative,
            from: next()?,
            to: next()?,
            length: next()?,
        })
    }
}

/// Error returned when parsing [`Mapping`].
#[derive(Error, Debug)]
pub enum ParseMappingError {
    /// [`Mapping`] requires 3 numbers.
    #[error("mapping must contain 3 numbers, given {count} numbers")]
    NotTriplet { count: usize },

    /// Number was not an integer.
    #[error("error parsing `{value}` as an integer")]
    ParseInt {
        source: ParseIntError,
        value: Box<str>,
    },
}

impl Display for Mapping {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Self {
            container_relative,
            from,
            to,
            length,
        } = *self;

        if container_relative {
            f.write_str("@")?;
        }

        write!(f, "{from}-{to}-{length}")
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use serde::de::value::UnitDeserializer;

    use super::*;

    #[test]
    fn uids() {
        let string = "uids=0-1-2#@3-4-5";
        let idmap: Idmap = string.parse().unwrap();

        assert_eq!(
            idmap,
            Idmap {
                uids: vec![
                    Mapping {
                        container_relative: false,
                        from: 0,
                        to: 1,
                        length: 2
                    },
                    Mapping {
                        container_relative: true,
                        from: 3,
                        to: 4,
                        length: 5
                    }
                ],
                gids: Vec::new(),
            },
        );
        assert_eq!(idmap.to_string(), string);
    }

    #[test]
    fn gids() {
        let string = "gids=0-1-2#@3-4-5";
        let idmap: Idmap = string.parse().unwrap();

        assert_eq!(
            idmap,
            Idmap {
                uids: Vec::new(),
                gids: vec![
                    Mapping {
                        container_relative: false,
                        from: 0,
                        to: 1,
                        length: 2
                    },
                    Mapping {
                        container_relative: true,
                        from: 3,
                        to: 4,
                        length: 5
                    }
                ],
            },
        );
        assert_eq!(idmap.to_string(), string);
    }

    #[test]
    fn uids_and_gids() {
        let string = "uids=0-1-2;gids=3-4-5";
        let idmap: Idmap = string.parse().unwrap();

        assert_eq!(
            idmap,
            Idmap {
                uids: vec![Mapping {
                    container_relative: false,
                    from: 0,
                    to: 1,
                    length: 2
                }],
                gids: vec![Mapping {
                    container_relative: false,
                    from: 3,
                    to: 4,
                    length: 5
                }],
            },
        );
        assert_eq!(idmap.to_string(), string);
    }

    #[test]
    fn deserialize_unit() {
        let idmap = Idmap::deserialize(UnitDeserializer::<de::value::Error>::new()).unwrap();
        assert_eq!(idmap, Idmap::default());
    }

    #[test]
    fn mapping_round_trip() {
        let string = "@0-1-2";
        let mapping: Mapping = string.parse().unwrap();

        assert_eq!(
            mapping,
            Mapping {
                container_relative: true,
                from: 0,
                to: 1,
                length: 2
            },
        );
        assert_eq!(mapping.to_string(), string);
    }
}
