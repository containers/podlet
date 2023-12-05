use std::fmt::Display;

use serde::{
    ser::{self, Impossible},
    Serialize,
};
use thiserror::Error;

/// Serializes a struct or map into arguments suitable for a shell.
///
/// # Errors
///
/// Returns an error if the value errors while serializing, the value is a non-serializable type,
/// the value has nested maps, or the value is a map without string keys.
///
/// ```
/// #[derive(Serialize)]
/// struct Example {
///     str: &'static str,
///     vec: Vec<u8>,
/// }
/// let example = Example {
///     str: "Hello world!",
///     vec: vec![1, 2],
/// };
/// assert_eq!(to_string(example).unwrap(), "--str \"Hello world!\" --vec 1 --vec 2");
/// ```
pub fn to_string<T: Serialize>(value: T) -> Result<String, Error> {
    let mut serializer = Serializer::default();
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("error while serializing args: {0}")]
    Custom(String),
    #[error("flag arg is missing")]
    MissingFlag,
    #[error("flag arg (map key or struct field) cannot be empty")]
    EmptyFlag,
    #[error("flags cannot be nested")]
    NestedFlag,
    #[error("map keys must a string")]
    InvalidMapKeyType,
    #[error("this type cannot be serialized")]
    InvalidType,
}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Custom(msg.to_string())
    }
}

/// A serializer for converting structs or maps into a series of flags and arguments.
///
/// Sequences are serialized by repeating the previous flag, saved in `current_flag`.
/// Values are responsible for adding the current flag as well as themselves to `output`.
/// `current_flag` is set when serializing map keys and struct fields.
#[derive(Default)]
struct Serializer {
    output: String,
    current_flag: Option<Box<str>>,
}

impl Serializer {
    /// Set the current flag.
    ///
    /// # Errors
    ///
    /// Returns an error if the given flag is empty or there is already a stored flag.
    fn set_current_flag(&mut self, flag: impl Into<Box<str>>) -> Result<(), Error> {
        let flag: Box<str> = flag.into();
        if flag.is_empty() {
            Err(Error::EmptyFlag)
        } else if self.current_flag.is_some() {
            Err(Error::NestedFlag)
        } else {
            self.current_flag = Some(flag);
            Ok(())
        }
    }

    /// Push the current flag into the `output`.
    ///
    /// # Errors
    ///
    /// Returns an error if there is not a current flag.
    fn push_flag(&mut self) -> Result<(), Error> {
        let flag = self.current_flag.as_deref().ok_or(Error::MissingFlag)?;
        if !self.output.is_empty() {
            self.output.push(' ');
        }
        self.output.push_str("--");
        self.output.push_str(flag);
        Ok(())
    }

    /// Pushes a `value` into `output`, see [`Serializer::push_value()`].
    fn push_display_value(&mut self, value: impl Display) -> Result<(), Error> {
        self.push_value(&value.to_string())
    }

    /// Pushes the current flag and a `value` into `output`.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error when pushing the current flag.
    fn push_value(&mut self, value: &str) -> Result<(), Error> {
        self.push_flag()?;
        if !value.is_empty() {
            self.output.push(' ');
            self.output.push_str(value);
        }
        Ok(())
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Self;

    type SerializeTuple = Self;

    type SerializeTupleStruct = Self;

    type SerializeTupleVariant = Self;

    type SerializeMap = Self;

    type SerializeStruct = Self;

    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        if v {
            self.push_flag()
        } else {
            self.push_value("false")
        }
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.push_display_value(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.push_display_value(v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.push_display_value(v)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.push_display_value(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.push_display_value(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.push_display_value(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.push_display_value(v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.push_display_value(v)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.push_display_value(v)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.push_display_value(v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.push_flag()?;
        self.output.push(' ');
        self.output.push(v);
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.push_value(&shlex::quote(v))
    }

    fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidType)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(self)
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        self.push_display_value(v)
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        self.push_display_value(v)
    }
}

impl ser::SerializeSeq for &mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.current_flag = None;
        Ok(())
    }
}

impl ser::SerializeTuple for &mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for &mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleVariant for &mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeMap for &mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        key.serialize(MapKeySerializer(self))
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)?;
        self.current_flag = None;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

struct MapKeySerializer<'a>(&'a mut Serializer);

impl<'a> ser::Serializer for MapKeySerializer<'a> {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Impossible<(), Error>;

    type SerializeTuple = Impossible<(), Error>;

    type SerializeTupleStruct = Impossible<(), Error>;

    type SerializeTupleVariant = Impossible<(), Error>;

    type SerializeMap = Impossible<(), Error>;

    type SerializeStruct = Impossible<(), Error>;

    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_bool(self, _: bool) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidMapKeyType)
    }

    fn serialize_i8(self, _: i8) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidMapKeyType)
    }

    fn serialize_i16(self, _: i16) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidMapKeyType)
    }

    fn serialize_i32(self, _: i32) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidMapKeyType)
    }

    fn serialize_i64(self, _: i64) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidMapKeyType)
    }

    fn serialize_u8(self, _: u8) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidMapKeyType)
    }

    fn serialize_u16(self, _: u16) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidMapKeyType)
    }

    fn serialize_u32(self, _: u32) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidMapKeyType)
    }

    fn serialize_u64(self, _: u64) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidMapKeyType)
    }

    fn serialize_f32(self, _: f32) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidMapKeyType)
    }

    fn serialize_f64(self, _: f64) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidMapKeyType)
    }

    fn serialize_char(self, _: char) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidMapKeyType)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.0.set_current_flag(v)
    }

    fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidMapKeyType)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::EmptyFlag)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::EmptyFlag)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidMapKeyType)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error::InvalidMapKeyType)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::InvalidMapKeyType)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::InvalidMapKeyType)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::InvalidMapKeyType)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Error::InvalidMapKeyType)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Error::InvalidMapKeyType)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::InvalidMapKeyType)
    }
}

impl ser::SerializeStruct for &mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.set_current_flag(key)?;
        value.serialize(&mut **self)?;
        self.current_flag = None;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl ser::SerializeStructVariant for &mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        ser::SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeStruct::end(self)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use indexmap::IndexMap;

    use super::*;

    #[test]
    fn basic_struct() {
        #[derive(Serialize)]
        #[serde(rename_all = "kebab-case")]
        struct Test {
            option_one: &'static str,
            two: u8,
        }

        let sut = Test {
            option_one: "one",
            two: 2,
        };
        assert_eq!(to_string(sut).unwrap(), "--option-one one --two 2");
    }

    #[test]
    fn struct_with_sequence() {
        #[derive(Serialize)]
        struct Test {
            tuple: (u16, u32, u64),
            array: [char; 3],
            vec: Vec<&'static str>,
        }

        let sut = Test {
            tuple: (1, 2, 3),
            array: ['a', 'b', 'c'],
            vec: vec!["one", "two", "three"],
        };
        assert_eq!(
            to_string(sut).unwrap(),
            "--tuple 1 --tuple 2 --tuple 3 \
             --array a --array b --array c \
             --vec one --vec two --vec three"
        );
    }

    #[test]
    fn map() {
        let map = IndexMap::from([("one", "1"), ("two", "2")]);
        assert_eq!(to_string(map).unwrap(), "--one 1 --two 2");

        let map = IndexMap::from([("one", ["1", "2"]), ("two", ["3", "4"])]);
        assert_eq!(to_string(map).unwrap(), "--one 1 --one 2 --two 3 --two 4");
    }

    #[test]
    fn escape_values() {
        let map = IndexMap::from([("one", "Hello, world!")]);
        assert_eq!(to_string(map).unwrap(), r#"--one "Hello, world!""#);
    }

    #[test]
    fn bool() {
        #[derive(Serialize)]
        struct Test {
            yes: bool,
            no: bool,
        }

        let sut = Test {
            yes: true,
            no: false,
        };
        assert_eq!(to_string(sut).unwrap(), "--yes --no false");
    }

    #[test]
    fn enum_value() {
        #[derive(Serialize)]
        #[serde(rename_all = "kebab-case")]
        enum Enum {
            One,
            Two,
        }

        #[derive(Serialize)]
        struct Test {
            one: Enum,
            two: Enum,
        }

        let sut = Test {
            one: Enum::One,
            two: Enum::Two,
        };

        assert_eq!(to_string(sut).unwrap(), "--one one --two two");
    }

    #[test]
    fn nested_err() {
        #[derive(Serialize)]
        struct Test {
            map: IndexMap<&'static str, &'static str>,
        }
        let sut = Test {
            map: IndexMap::from([("one", "1"), ("two", "2")]),
        };
        assert_eq!(to_string(sut).unwrap_err(), Error::NestedFlag);
    }
}
