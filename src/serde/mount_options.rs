//! (De)serialization for structs representing mount options.

use std::fmt::Display;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use self::{de::Deserializer, ser::Serializer};

mod de;
mod ser;

/// Serializes `value` to a string using a serializer designed
/// for structs that represent mount options.
///
/// # Errors
///
/// Returns an error if the value errors while serializing, or if given
/// an invalid type, such as a non-struct or a struct with a nested map or sequence.
///
/// # Example
///
/// ```
/// use serde::Serialize;
///
/// #[derive(Serialize, PartialEq)]
/// #[serde(rename_all = "kebab-case")]
/// struct Example<'a> {
///     hello: &'a str,
///     option: bool,
/// }
/// let example = Example {
///     hello: "world",
///     option: false,
/// };
/// assert_eq!(
///     to_string(example).unwrap(),
///     "hello=world,option=false",
/// );
/// ```
pub fn to_string<T: Serialize>(value: T) -> Result<String, Error> {
    let mut serializer = Serializer::default();
    value.serialize(&mut serializer)?;
    Ok(serializer.into_output())
}

/// Deserializes a struct from a string using a deserializer designed
/// for structs that represent mount options.
///
/// # Errors
///
/// Returns an error if the value errors while deserializing, or when attempting to deserialize
/// an invalid type, such as a non-struct or a struct with a nested map or sequence.
///
/// # Example
///
/// ```
/// use serde::Deserialize;
///
/// #[derive(Deserialize, PartialEq)]
/// #[serde(rename_all = "kebab-case")]
/// struct Example<'a> {
///     hello: &'a str,
///     option: bool,
/// }
/// assert_eq!(
///     from_str::<Example>("hello=world,option").unwrap(),
///     Example {
///         hello: "world",
///         option: true,
///     },
/// );
/// ```
pub fn from_str<'de, T: Deserialize<'de>>(input: &'de str) -> Result<T, Error> {
    let mut deserializer = Deserializer::from_str(input);
    T::deserialize(&mut deserializer)
}

/// Error returned when using [`Serializer`] or [`Deserializer`].
#[derive(Error, Debug)]
pub enum Error {
    /// An error occurred while (de)serializing.
    #[error("error while (de)serializing: {0}")]
    Custom(String),

    /// A type that cannot be (de)serialized was encountered.
    #[error("type cannot be (de)serialized")]
    BadType,
}

impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Self::Custom(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        let msg = msg.to_string().replace("field", "option");
        Self::Custom(msg)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    #[serde(rename_all = "kebab-case")]
    struct Test<'a> {
        bool: bool,
        pos_int: u8,
        neg_int: i8,
        float: f32,
        char: char,
        str: &'a str,
        string: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        some: Option<()>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        none: Option<()>,
        unit: (),
        unit_enum: Enum,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    #[serde(rename_all = "kebab-case")]
    enum Enum {
        One,
        Two,
    }

    impl Test<'static> {
        const DEFAULT: &'static str = "bool=true,pos-int=42,neg-int=-42,float=4.2,char=a,\
            str=hello,string=world,some,unit,unit-enum=one";

        fn new() -> Self {
            Self {
                bool: true,
                pos_int: 42,
                neg_int: -42,
                float: 4.2,
                char: 'a',
                str: "hello",
                string: String::from("world"),
                some: Some(()),
                none: None,
                unit: (),
                unit_enum: Enum::One,
            }
        }
    }

    #[test]
    fn serialize() {
        assert_eq!(to_string(Test::new()).unwrap(), Test::DEFAULT);
    }

    #[test]
    fn deserialize() {
        let test: Test = from_str(Test::DEFAULT).unwrap();
        assert_eq!(test, Test::new());
    }

    #[test]
    fn deserialize_bool() {
        #[derive(Deserialize, Debug, PartialEq, Eq)]
        struct Test {
            #[serde(default)]
            bool: bool,
        }

        let test: Test = from_str("bool=true").unwrap();
        assert_eq!(test, Test { bool: true });

        let test: Test = from_str("bool=false").unwrap();
        assert_eq!(test, Test { bool: false });

        let test: Test = from_str("bool").unwrap();
        assert_eq!(test, Test { bool: true });

        let test: Test = from_str("").unwrap();
        assert_eq!(test, Test { bool: false });
    }
}
