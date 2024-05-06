use std::fmt::{Display, Write};

use serde::{
    ser::{self, Impossible},
    Serialize,
};
use thiserror::Error;

/// Alias for `quote_spaces_join::<' ', T, S>()`.
pub fn quote_spaces_join_space<'a, T, S>(iter: &'a T, serializer: S) -> Result<S::Ok, S::Error>
where
    &'a T: IntoIterator,
    <&'a T as IntoIterator>::Item: AsRef<str>,
    S: ser::Serializer,
{
    quote_spaces_join::<' ', _, _>(iter, serializer)
}

/// Alias for `quote_spaces_join::<':', T, S>()`.
pub fn quote_spaces_join_colon<'a, T, S>(iter: &'a T, serializer: S) -> Result<S::Ok, S::Error>
where
    &'a T: IntoIterator,
    <&'a T as IntoIterator>::Item: AsRef<str>,
    S: ser::Serializer,
{
    quote_spaces_join::<':', _, _>(iter, serializer)
}

/// Serializes `iter` by joining its items separated by `C`.
/// Each item is quoted if it has spaces before being joined.
///
/// For example, `["one", "two three", "four"]`, if C = ' '
/// is serialized as `one "two three" four`.
pub fn quote_spaces_join<'a, const C: char, T, S>(
    iter: &'a T,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    &'a T: IntoIterator,
    <&'a T as IntoIterator>::Item: AsRef<str>,
    S: ser::Serializer,
{
    let mut output = String::new();
    let mut iter = iter.into_iter();

    if let Some(first) = iter.next() {
        quote_spaces_push(&mut output, first.as_ref());
    }

    for item in iter {
        output.push(C);
        quote_spaces_push(&mut output, item.as_ref());
    }

    output.serialize(serializer)
}

/// Appends `item` to `output`, quoting it if it contains spaces.
fn quote_spaces_push(output: &mut String, item: &str) {
    if item.contains(char::is_whitespace) {
        output.push('"');
        for char in item.chars() {
            match char {
                '\n' => output.push_str(r"\n"),
                _ => output.push(char),
            }
        }
        output.push('"');
    } else {
        output.push_str(item);
    }
}

/// Serializes `value` to a string using a serializer designed
/// for structs that represent a section of a quadlet file.
///
/// # Errors
///
/// Returns an error if the value errors while serializing, or if given
/// an invalid type, such as a non-struct or a struct with a nested map.
///
/// ```
/// #[derive(Serialize)]
/// #[serde(rename_all = "PascalCase")]
/// struct Example {
///     str: &'static str,
///     vec: Vec<u8>,
/// }
/// let example = Example {
///     str: "Hello world!",
///     vec: vec![1, 2],
/// };
/// assert_eq!(
///     to_string(example).unwrap(),
///     "[Example]\n\
///     Str=Hello world!\n\
///     Vec=1\n\
///     Vec=2\n"
/// );
/// ```
pub fn to_string<T: Serialize>(value: T) -> Result<String, Error> {
    let mut serializer = Serializer::default();
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

/// The same as [`to_string()`] except the table name is not included.
///
/// ```
/// #[derive(Serialize)]
/// #[serde(rename_all = "PascalCase")]
/// struct Example {
///     str: &'static str,
///     vec: Vec<u8>,
/// }
/// let example = Example {
///     str: "Hello world!",
///     vec: vec![1, 2],
/// };
/// assert_eq!(
///     to_string_no_table_name(example).unwrap(),
///     "Str=Hello world!\n\
///     Vec=1\n\
///     Vec=2\n"
/// );
/// ```
pub fn to_string_no_table_name<T: Serialize>(value: T) -> Result<String, Error> {
    let mut serializer = Serializer {
        output: String::new(),
        no_table_name: true,
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("error while serializing: {0}")]
    Custom(String),
    #[error("type cannot be serialized")]
    InvalidType,
}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Custom(msg.to_string())
    }
}

/// A serializer for converting structs to quadlet file sections.
#[derive(Default)]
struct Serializer {
    output: String,
    no_table_name: bool,
}

impl ser::Serializer for &mut Serializer {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Impossible<(), Error>;

    type SerializeTuple = Impossible<(), Error>;

    type SerializeTupleStruct = Impossible<(), Error>;

    type SerializeTupleVariant = Impossible<(), Error>;

    type SerializeMap = Impossible<(), Error>;

    type SerializeStruct = Self;

    type SerializeStructVariant = Self;

    serialize_invalid_primitives! {
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
        Err(Error::InvalidType)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidType)
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
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidType)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidType)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidType)
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
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        if !self.no_table_name {
            writeln!(self.output, "[{name}]").expect("write to String never fails");
        }
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        if !self.no_table_name {
            writeln!(self.output, "[{variant}]").expect("write to String never fails");
        }
        Ok(self)
    }
}

impl ser::SerializeStruct for &mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut ValueSerializer {
            serializer: self,
            key,
        })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl ser::SerializeStructVariant for &mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeStruct::end(self)
    }
}

/// Serializes values for [`Serializer`].
///
/// Sequences are serialized on separate lines, repeating the `key`.
struct ValueSerializer<'a> {
    serializer: &'a mut Serializer,
    key: &'static str,
}

impl<'a> ValueSerializer<'a> {
    /// Writes the `value` to `serializer.output` as `key=value`.
    fn write_value(&mut self, value: impl Display) {
        writeln!(self.serializer.output, "{}={value}", self.key)
            .expect("write to String never fails");
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
        write_value,
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
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidType)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
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

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
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
        Err(Error::InvalidType)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Error::InvalidType)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::InvalidType)
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Display,
    {
        self.write_value(value);
        Ok(())
    }
}

impl<'a> ser::SerializeSeq for &mut ValueSerializer<'a> {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &mut ValueSerializer<'a> {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a> ser::SerializeTupleStruct for &mut ValueSerializer<'a> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a> ser::SerializeTupleVariant for &mut ValueSerializer<'a> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn basic_struct() {
        #[derive(Serialize)]
        #[serde(rename_all = "PascalCase")]
        struct Test {
            one: u8,
            two: &'static str,
        }

        let sut = Test { one: 1, two: "two" };
        assert_eq!(
            to_string(sut).unwrap(),
            "[Test]\n\
            One=1\n\
            Two=two\n"
        );
    }

    #[test]
    fn sequence() {
        #[derive(Serialize)]
        #[serde(rename_all = "PascalCase")]
        struct Test {
            vec: Vec<u8>,
        }

        let sut = Test { vec: vec![1, 2, 3] };
        assert_eq!(
            to_string(sut).unwrap(),
            "[Test]\n\
            Vec=1\n\
            Vec=2\n\
            Vec=3\n"
        );
    }

    #[test]
    fn sequence_join() {
        #[derive(Serialize)]
        #[serde(rename_all = "PascalCase")]
        struct Test {
            #[serde(serialize_with = "quote_spaces_join_space")]
            vec: Vec<&'static str>,
        }

        let sut = Test {
            vec: vec!["one", "two three", "four"],
        };
        assert_eq!(
            to_string(sut).unwrap(),
            "[Test]\n\
            Vec=one \"two three\" four\n"
        );
    }

    #[test]
    fn empty_is_empty() {
        #[derive(Serialize, Default)]
        #[serde(rename_all = "PascalCase")]
        struct Test {
            option: Option<&'static str>,
            vec: Vec<&'static str>,
            #[serde(
                serialize_with = "quote_spaces_join_space",
                skip_serializing_if = "Vec::is_empty"
            )]
            vec_joined: Vec<&'static str>,
        }

        assert_eq!(to_string(Test::default()).unwrap(), "[Test]\n");
    }

    #[test]
    fn nested_map_err() {
        #[derive(Serialize)]
        #[serde(rename_all = "PascalCase")]
        struct Test {
            map: HashMap<&'static str, &'static str>,
        }

        let sut = Test {
            map: [("one", "one")].into(),
        };
        assert_eq!(to_string(sut).unwrap_err(), Error::InvalidType);
    }

    #[test]
    fn quote_spaces() {
        let mut output = String::new();
        quote_spaces_push(&mut output, "test1 test2");
        assert_eq!(output, r#""test1 test2""#);

        output.clear();
        quote_spaces_push(&mut output, "test1\ntest2");
        assert_eq!(output, r#""test1\ntest2""#);
    }
}
