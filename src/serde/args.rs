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
    #[error("flag cannot be empty or contain whitespace")]
    InvalidFlag,
    #[error("cannot serialize structs with nested structs or maps")]
    Nested,
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

/// A serializer for converting structs into a series of flags and arguments.
///
/// Sequences are serialized by repeating the previous flag.
/// Values are responsible for adding the current flag as well as themselves to `output`.
#[derive(Default)]
struct Serializer {
    output: String,
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Impossible<(), Error>;

    type SerializeTuple = Impossible<(), Error>;

    type SerializeTupleStruct = Impossible<(), Error>;

    type SerializeTupleVariant = Impossible<(), Error>;

    type SerializeMap = Impossible<(), Error>;

    type SerializeStruct = Self;

    type SerializeStructVariant = Self;

    invalid_primitives! {
        Error::InvalidType,
        serialize_bool: bool,
        serialize_i8: i8,
        serialize_i16: i16,
        serialize_i32: i32,
        serialize_i64: i64,
        serialize_i128: i128,
        serialize_u8: u8,
        serialize_u16: u16,
        serialize_u32: u32,
        serialize_u64: u64,
        serialize_u128: u128,
        serialize_f32: f32,
        serialize_f64: f64,
        serialize_char: char,
        serialize_str: &str,
        serialize_bytes: &[u8],
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
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(())
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
        Err(Error::InvalidType)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::InvalidType)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::InvalidType)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::InvalidType)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Error::InvalidType)
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
        if key.is_empty() || key.contains(char::is_whitespace) {
            Err(Error::InvalidFlag)
        } else {
            value.serialize(&mut ValueSerializer {
                serializer: self,
                flag: key,
            })
        }
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

/// Serializes values for [`Serializer`].
///
/// Sequences are serialized by repeating the flag.
struct ValueSerializer<'a> {
    serializer: &'a mut Serializer,
    flag: &'static str,
}

impl<'a> ValueSerializer<'a> {
    /// Append `--{flag}` to `serializer.output`.
    fn push_flag(&mut self) {
        let output = &mut self.serializer.output;
        if !output.is_empty() {
            output.push(' ');
        }
        output.push_str("--");
        output.push_str(self.flag);
    }

    /// Append `--{flag} {arg}` to `serializer.output`.
    fn push_arg(&mut self, arg: &str) {
        self.push_flag();
        if !arg.is_empty() {
            let output = &mut self.serializer.output;
            output.push(' ');
            output.push_str(&shlex::quote(arg));
        }
    }

    /// Append `--{flag} {arg}` to `serializer.output`.
    fn push_display_arg(&mut self, arg: impl Display) {
        self.push_arg(&arg.to_string());
    }
}

impl<'a> ser::Serializer for &mut ValueSerializer<'a> {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Self;

    type SerializeTuple = Self;

    type SerializeTupleStruct = Self;

    type SerializeTupleVariant = Self;

    type SerializeMap = Impossible<(), Error>;

    type SerializeStruct = Impossible<(), Error>;

    type SerializeStructVariant = Impossible<(), Error>;

    serialize_primitives! {
        push_display_arg,
        serialize_i8: i8,
        serialize_i16: i16,
        serialize_i32: i32,
        serialize_i64: i64,
        serialize_i128: i128,
        serialize_u8: u8,
        serialize_u16: u16,
        serialize_u32: u32,
        serialize_u64: u64,
        serialize_u128: u128,
        serialize_f32: f32,
        serialize_f64: f64,
    }

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        if v {
            self.push_flag();
        } else {
            self.push_flag();
            self.serializer.output.push_str("=false");
        }
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.push_flag();
        self.serializer.output.push(' ');
        self.serializer.output.push(v);
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.push_arg(v);
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
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
        Err(Error::InvalidType)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidType)
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
        Err(Error::Nested)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Error::Nested)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::Nested)
    }
}

impl<'a> ser::SerializeSeq for &mut ValueSerializer<'a> {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &mut ValueSerializer<'a> {
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

impl<'a> ser::SerializeTupleStruct for &mut ValueSerializer<'a> {
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

impl<'a> ser::SerializeTupleVariant for &mut ValueSerializer<'a> {
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
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
    fn escape_values() {
        #[derive(Serialize)]
        struct Test {
            test: &'static str,
        }

        let sut = Test {
            test: "Hello, world!",
        };
        assert_eq!(to_string(sut).unwrap(), r#"--test "Hello, world!""#);
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
        assert_eq!(to_string(sut).unwrap(), "--yes --no=false");
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
            nested: Nested,
        }

        #[derive(Serialize)]
        struct Nested {
            nested: &'static str,
        }

        let sut = Test {
            nested: Nested { nested: "nested" },
        };
        assert_eq!(to_string(sut).unwrap_err(), Error::Nested);
    }

    #[test]
    fn invalid_flag_err() {
        #[derive(Serialize)]
        struct Test1 {
            #[serde(rename = "")]
            empty: (),
        }

        #[derive(Serialize)]
        struct Test2 {
            #[serde(rename = "hello world")]
            spaces: (),
        }

        let sut = Test1 { empty: () };
        assert_eq!(to_string(sut).unwrap_err(), Error::InvalidFlag);

        let sut = Test2 { spaces: () };
        assert_eq!(to_string(sut).unwrap_err(), Error::InvalidFlag);
    }
}
