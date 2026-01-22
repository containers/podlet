//! Provides [`Tmpfs`] for [`Mount::Tmpfs`](super::Mount::Tmpfs) and
//! [`Mount::Ramfs`](super::Mount::Ramfs).

use std::{
    fmt::{self, Display, Formatter},
    num::ParseIntError,
    path::PathBuf,
    str::FromStr,
};

use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, MapAccess},
    ser::SerializeStruct,
};
use thiserror::Error;
use umask::{Mode, STICKY};

use super::mode;

/// Default [`Mode`] for [`Tmpfs`]: `0o1777`.
const MODE_DEFAULT: Mode = Mode::all().with_extra(STICKY);

/// tmpfs and ramfs type [`Mount`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tmpfs {
    /// Mount destination spec.
    // #[serde(alias = "dst", alias = "target")]
    pub destination: PathBuf,

    /// Read only permission.
    // #[serde(
    //     default,
    //     rename = "readonly",
    //     alias = "ro",
    //     skip_serializing_if = "skip_default"
    // )]
    pub read_only: bool,

    /// Size of the tmpfs/ramfs mount in bytes.
    ///
    /// Unlimited by default in Linux.
    // #[serde(default, rename = "tmpfs-size", skip_serializing_if = "skip_default")]
    pub size: Size,

    /// File mode of the tmpfs/ramfs (default 1777).
    // #[serde(
    //     default = 0o1777,
    //     rename = "tmpfs-mode",
    //     with = "mode",
    //     skip_serializing_if = "skip_default"
    // )]
    pub mode: Mode,

    /// Enable copyup from the image directory at the same location to the tmpfs/ramfs.
    ///
    /// Enabled by default.
    ///
    /// `notmpcopyup` deserialized as `false`. If `false`, `notmpcopyup` is serialized as unit `()`.
    // #[serde(default = true, skip_serializing_if = "skip_true")]
    pub tmpcopyup: bool,

    /// Change recursively the owner and group of the source volume based on the UID and GID of the
    /// container.
    // #[serde(default, skip_serializing_if = "Not::not")]
    pub chown: bool,
}

impl Tmpfs {
    /// Create a new [`Tmpfs`] from a destination with defaults.
    #[cfg(test)]
    fn new(destination: PathBuf) -> Self {
        Self {
            destination,
            read_only: false,
            size: Size::default(),
            mode: MODE_DEFAULT,
            tmpcopyup: true,
            chown: false,
        }
    }
}

impl Serialize for Tmpfs {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let Self {
            ref destination,
            read_only,
            size,
            mode,
            tmpcopyup,
            chown,
        } = *self;

        // destination + non-defaults
        let len = 1
            + usize::from(read_only)
            + usize::from(size != Size::default())
            + usize::from(mode != MODE_DEFAULT)
            + usize::from(!tmpcopyup)
            + usize::from(chown);

        let mut state = serializer.serialize_struct("Tmpfs", len)?;

        state.serialize_field(Field::Destination.as_str(), destination)?;

        if read_only {
            state.serialize_field(Field::ReadOnly.as_str(), &read_only)?;
        } else {
            state.skip_field(Field::Destination.as_str())?;
        }

        if size == Size::default() {
            state.skip_field(Field::Size.as_str())?;
        } else {
            state.serialize_field(Field::Size.as_str(), &size)?;
        }

        if mode == MODE_DEFAULT {
            state.skip_field(Field::Mode.as_str())?;
        } else {
            // serde(with = "mode")
            state.serialize_field(Field::Mode.as_str(), &SerdeMode(mode))?;
        }

        // tmpcopyup, notmpcopyup values (de)serialize as unit (i.e. no value)
        if tmpcopyup {
            state.skip_field(Field::NoTmpCopyUp.as_str())?;
        } else {
            state.serialize_field(Field::NoTmpCopyUp.as_str(), &())?;
        }
        state.skip_field(Field::TmpCopyUp.as_str())?;

        if chown {
            state.serialize_field(Field::Chown.as_str(), &chown)?;
        } else {
            state.skip_field(Field::Chown.as_str())?;
        }

        state.end()
    }
}

impl<'de> Deserialize<'de> for Tmpfs {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_struct("Tmpfs", Field::FIELDS, Visitor)
    }
}

struct Visitor;

impl<'de> de::Visitor<'de> for Visitor {
    type Value = Tmpfs;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("tmpfs mount options")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut destination = None;
        let mut read_only = false;
        let mut size = None;
        let mut mode = None;
        let mut tmpcopyup = true;
        let mut chown = false;

        while let Some(field) = map.next_key()? {
            match field {
                Field::Destination => {
                    check_duplicate(destination.as_ref(), Field::Destination)?;
                    destination = Some(map.next_value()?);
                }
                Field::ReadOnly => {
                    // "ro" is equivalent to "ro=true"
                    let value: Option<bool> = map.next_value()?;
                    read_only = value.unwrap_or(true);
                }
                Field::Size => {
                    check_duplicate(size.as_ref(), Field::Size)?;
                    size = Some(map.next_value()?);
                }
                Field::Mode => {
                    check_duplicate(mode.as_ref(), Field::Mode)?;
                    // serde(with = "mode")
                    let SerdeMode(value) = map.next_value()?;
                    mode = Some(value);
                }
                // tmpcopyup, notmpcopyup values (de)serialize as unit (i.e. no value)
                Field::TmpCopyUp => tmpcopyup = true,
                Field::NoTmpCopyUp => tmpcopyup = false,
                Field::Chown => {
                    // Similar as read_only
                    let value: Option<bool> = map.next_value()?;
                    chown = value.unwrap_or(true);
                }
            }
        }

        // only destination is required
        let destination =
            destination.ok_or_else(|| de::Error::missing_field(Field::Destination.as_str()))?;

        Ok(Tmpfs {
            destination,
            read_only,
            size: size.unwrap_or_default(),
            mode: mode.unwrap_or(MODE_DEFAULT),
            tmpcopyup,
            chown,
        })
    }
}

/// Check is `option` is already set.
///
/// # Errors
///
/// Returns a [duplicate field](de::Error::duplicate_field()) error if `option` is [`Some`].
fn check_duplicate<T, E: de::Error>(option: Option<&T>, field: Field) -> Result<(), E> {
    if option.is_some() {
        Err(de::Error::duplicate_field(field.as_str()))
    } else {
        Ok(())
    }
}

/// Transparent (de)serialize wrapper for [`Mode`].
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
struct SerdeMode(#[serde(with = "mode")] Mode);

/// Fields of [`Tmpfs`].
#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(field_identifier, rename_all = "kebab-case", deny_unknown_fields)]
enum Field {
    /// Mount destination spec.
    #[serde(alias = "dst", alias = "target")]
    Destination,

    /// Read only permission.
    #[serde(rename = "readonly", alias = "ro")]
    ReadOnly,

    /// Size of the tmpfs/ramfs mount in bytes.
    ///
    /// Unlimited by default in Linux.
    #[serde(rename = "tmpfs-size")]
    Size,

    /// File mode of the tmpfs/ramfs (default 1777).
    #[serde(rename = "tmpfs-mode")]
    Mode,

    /// Enable copyup from the image directory at the same location to the tmpfs/ramfs.
    ///
    /// Enabled by default.
    #[serde(rename = "tmpcopyup")]
    TmpCopyUp,

    /// Disable copying files from the image to the tmpfs/ramfs.
    #[serde(rename = "notmpcopyup")]
    NoTmpCopyUp,

    /// Change recursively the owner and group of the source volume based on the UID and GID of the
    /// container.
    #[serde(alias = "U")]
    Chown,
}

impl Field {
    /// All [`Field`]s.
    const FIELDS: &'static [&'static str] = &[
        Self::Destination.as_str(),
        Self::ReadOnly.as_str(),
        Self::Size.as_str(),
        Self::Mode.as_str(),
        Self::TmpCopyUp.as_str(),
        Self::NoTmpCopyUp.as_str(),
        Self::Chown.as_str(),
    ];

    /// Field name as a string slice.
    const fn as_str(self) -> &'static str {
        match self {
            Self::Destination => "destination",
            Self::ReadOnly => "readonly",
            Self::Size => "tmpfs-size",
            Self::Mode => "tmpfs-mode",
            Self::TmpCopyUp => "tmpcopyup",
            Self::NoTmpCopyUp => "notmpcopyup",
            Self::Chown => "chown",
        }
    }
}

/// Size in bytes or percentage
///
/// (De)serializes from/to a string with a suffix, e.g. "200m" or "50%".
/// [`Unlimited`](Size::Unlimited) (de)serializes from/to an empty string.
///
/// See the `size` mount option of
/// [**tmpfs(5)**](https://man7.org/linux/man-pages/man5/tmpfs.5.html).
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    Bytes(u64),
    Kibibytes(u64),
    Mebibytes(u64),
    Gibibytes(u64),
    Percent(u8),
    #[default]
    Unlimited,
}

impl Display for Size {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Bytes(bytes) => Display::fmt(bytes, f),
            Self::Kibibytes(kib) => write!(f, "{kib}k"),
            Self::Mebibytes(mib) => write!(f, "{mib}m"),
            Self::Gibibytes(gib) => write!(f, "{gib}g"),
            Self::Percent(percent) => write!(f, "{percent}%"),
            Self::Unlimited => Ok(()),
        }
    }
}

impl FromStr for Size {
    type Err = ParseSizeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // suffix can be %, g/G, m/M, or k/K
        if s.is_empty() {
            Ok(Self::Unlimited)
        } else if let Some(percent) = s.strip_suffix('%') {
            percent
                .parse()
                .map(Self::Percent)
                .map_err(|source| ParseSizeError {
                    source,
                    value: percent.to_owned(),
                })
        } else if let Some(gib) = s.strip_suffix(['g', 'G']) {
            gib.parse()
                .map(Self::Gibibytes)
                .map_err(|source| ParseSizeError {
                    source,
                    value: gib.to_owned(),
                })
        } else if let Some(mib) = s.strip_suffix(['m', 'M']) {
            mib.parse()
                .map(Self::Mebibytes)
                .map_err(|source| ParseSizeError {
                    source,
                    value: mib.to_owned(),
                })
        } else if let Some(kib) = s.strip_suffix(['k', 'K']) {
            kib.parse()
                .map(Self::Kibibytes)
                .map_err(|source| ParseSizeError {
                    source,
                    value: kib.to_owned(),
                })
        } else {
            s.parse().map(Self::Bytes).map_err(|source| ParseSizeError {
                source,
                value: s.to_owned(),
            })
        }
    }
}

/// Error returned when parsing [`Size`].
#[derive(Error, Debug)]
#[error("size must be an integer: {value}")]
pub struct ParseSizeError {
    source: ParseIntError,
    value: String,
}

impl Serialize for Size {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Size {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let size: &str = Deserialize::deserialize(deserializer)?;
        size.parse().map_err(de::Error::custom)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{super::Mount, *};

    #[test]
    fn mode_default() {
        assert_eq!(MODE_DEFAULT, Mode::from(0o1777));
    }

    #[test]
    fn defaults() {
        let string = "type=tmpfs,destination=/test";
        let mount: Mount = string.parse().unwrap();

        assert_eq!(mount, Mount::Tmpfs(Tmpfs::new("/test".into())));
        assert_eq!(mount.to_string(), string);
    }

    #[test]
    fn tmpcopyup() {
        let mount: Mount = "type=tmpfs,destination=/test,tmpcopyup".parse().unwrap();

        // tmpcopyup default is true
        assert_eq!(mount, Mount::Tmpfs(Tmpfs::new("/test".into())));

        let string = "type=tmpfs,destination=/test,notmpcopyup";
        let mount: Mount = string.parse().unwrap();

        assert_eq!(
            mount,
            Mount::Tmpfs(Tmpfs {
                tmpcopyup: false,
                ..Tmpfs::new("/test".into())
            }),
        );
        assert_eq!(mount.to_string(), string);
    }
}
