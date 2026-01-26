//! (De)serialize [`Mode`] as a string. For use in `#[serde(with = "mode")]`.

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use umask::{Mode, READ, USER, WRITE};

/// Serialize [`Mode`] as a string.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn serialize<S: Serializer>(mode: &Mode, serializer: S) -> Result<S::Ok, S::Error> {
    let mode = u32::from(mode);
    format_args!("{mode:o}").serialize(serializer)
}

/// Deserialize [`Mode`] from a string.
pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Mode, D::Error> {
    let mode = Deserialize::deserialize(deserializer)?;
    u32::from_str_radix(mode, 8)
        .map(Into::into)
        .map_err(de::Error::custom)
}

/// Default [`Mode`] for [`DevPts`](super::DevPts): `0o600`.
pub const fn default() -> Mode {
    Mode::new().with_class_perm(USER, READ | WRITE)
}

/// Skip serializing for default [`Mode`].
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn skip_default(mode: &Mode) -> bool {
    *mode == default()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::fmt::{self, Display, Formatter};

    use serde::de::value::{BorrowedStrDeserializer, Error};

    use super::*;

    #[test]
    fn devpts_default() {
        assert_eq!(default(), Mode::from(0o600));
    }

    #[test]
    fn serialize_string() {
        struct Test(Mode);

        impl Display for Test {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                serialize(&self.0, f)
            }
        }

        let test = Test(Mode::from(0o755));
        assert_eq!(test.to_string(), "755");
    }

    #[test]
    fn deserialize_string() {
        let mode = deserialize(BorrowedStrDeserializer::<Error>::new("755")).unwrap();
        assert_eq!(mode, Mode::from(0o755));
    }
}
