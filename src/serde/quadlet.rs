use std::{
    collections::HashSet,
    fmt::{self, Display, Formatter, Write},
};

use serde::{
    Serialize,
    ser::{self, Impossible},
};
use thiserror::Error;

use crate::quadlet::JoinOption;

/// Serialize a sequence of strings, adding quotes to each string that contains whitespace.
///
/// # Errors
///
/// Returns an error if the `serializer` does.
pub fn seq_quote_whitespace<'a, I, T, S>(iter: I, serializer: S) -> Result<S::Ok, S::Error>
where
    I: IntoIterator<Item = &'a T>,
    T: ?Sized + AsRef<str> + 'a,
    S: ser::Serializer,
{
    serializer.collect_seq(iter.into_iter().map(|item| QuoteWhitespace(item.as_ref())))
}

/// String slice wrapper that adds quotes in its [`Display`] impl if the string contains whitespace.
///
/// Newline characters are also rewritten to a literal "\n".
struct QuoteWhitespace<'a>(&'a str);

impl Display for QuoteWhitespace<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.0.contains(char::is_whitespace) {
            f.write_char('"')?;
            for char in self.0.chars() {
                match char {
                    '\n' => f.write_str(r"\n")?,
                    char => f.write_char(char)?,
                }
            }
            f.write_char('"')
        } else {
            f.write_str(self.0)
        }
    }
}

impl Serialize for QuoteWhitespace<'_> {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

/// Alias for [`to_string(value, &JoinOption::all_set())`](to_string()).
///
/// # Errors
///
/// See [`to_string()`].
#[cfg(test)]
pub fn to_string_join_all<T: Serialize>(value: T) -> Result<String, Error> {
    to_string(value, &JoinOption::all_set())
}

/// Serializes `value` to a string using a serializer designed for structs that represent a section
/// of a Quadlet file.
///
/// Only structs, sequences, and tuples are allowed at the top level. Elements in a sequence are
/// serialized as separate sections with a new line inserted between them. Tuples can be used to
/// combine structs into a single section; the name from the first type in the tuple is used as the
/// section name.
///
/// `join_options` should be a set of all Quadlet options for which sequence values should be joined
/// together by a space.
///
/// # Errors
///
/// Returns an error if the value errors while serializing, or if given an invalid type, such as a
/// struct with a nested map.
///
/// # Examples
///
/// ```
/// #[derive(Serialize)]
/// #[serde(rename_all = "PascalCase")]
/// struct Example {
///     str: &'static str,
///     vec: Vec<u8>,
///     #[serde(serialize_with = "seq_quote_whitespace")]
///     environment: Vec<&'static str>,
/// }
///
/// let example = Example {
///     str: "Hello world!",
///     vec: vec![1, 2],
///     environment: vec!["ONE=one", "TWO=two two"],
/// };
/// assert_eq!(
///     to_string(example, &[JoinOption::Environment].into())?,
///     "[Example]\n\
///     Str=Hello world!\n\
///     Vec=1\n\
///     Vec=2\n\
///     Environment=One=one \"TWO=two two\"\n"
/// );
/// # Ok::<(), Error>(())
/// ```
pub fn to_string<T>(value: T, join_options: &HashSet<JoinOption>) -> Result<String, Error>
where
    T: Serialize,
{
    let mut serializer = Serializer::new(join_options);
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

/// A serializer for converting structs to Quadlet file sections.
struct Serializer<'a> {
    output: String,
    skip_section_name: bool,
    join_options: &'a HashSet<JoinOption>,
}

impl<'a> Serializer<'a> {
    fn new(join_options: &'a HashSet<JoinOption>) -> Self {
        Self {
            output: String::new(),
            skip_section_name: false,
            join_options,
        }
    }
}

impl ser::Serializer for &mut Serializer<'_> {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Self;

    type SerializeTuple = Self;

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
        if !self.skip_section_name {
            writeln!(self.output, "[{name}]").expect("write to String never fails");
        }
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_struct(variant, len)
    }
}

impl ser::SerializeSeq for &mut Serializer<'_> {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        if !self.output.is_empty() {
            self.output.push('\n');
        }
        value.serialize(&mut (**self))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl ser::SerializeTuple for &mut Serializer<'_> {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut (**self))?;
        self.skip_section_name = true;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.skip_section_name = false;
        Ok(())
    }
}

impl ser::SerializeStruct for &mut Serializer<'_> {
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

impl ser::SerializeStructVariant for &mut Serializer<'_> {
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
/// Sequences are serialized on separate lines, repeating the `key`, unless the `key` is contained
/// in `serializer.join_options`. Nested sequences result in an error.
struct ValueSerializer<'a, 'b> {
    serializer: &'a mut Serializer<'b>,
    key: &'static str,
}

impl ValueSerializer<'_, '_> {
    /// Writes the `value` to `serializer.output` as `key=value`.
    fn write_value(&mut self, value: impl Display) {
        writeln!(self.serializer.output, "{}={value}", self.key)
            .expect("write to String never fails");
    }
}

impl<'a> ser::Serializer for &'a mut ValueSerializer<'a, '_> {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = SeqValueSerializer<'a>;

    type SerializeTuple = SeqValueSerializer<'a>;

    type SerializeTupleStruct = SeqValueSerializer<'a>;

    type SerializeTupleVariant = SeqValueSerializer<'a>;

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
        Ok(SeqValueSerializer::new(
            &mut self.serializer.output,
            self.key,
            self.serializer.join_options,
        ))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(SeqValueSerializer::new(
            &mut self.serializer.output,
            self.key,
            self.serializer.join_options,
        ))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(SeqValueSerializer::new(
            &mut self.serializer.output,
            self.key,
            self.serializer.join_options,
        ))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(SeqValueSerializer::new(
            &mut self.serializer.output,
            self.key,
            self.serializer.join_options,
        ))
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

    fn collect_str<T: ?Sized + Display>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        self.write_value(value);
        Ok(())
    }
}

/// Serializes sequence values for [`Serializer`]/[`ValueSerializer`].
struct SeqValueSerializer<'a> {
    /// The output of the [`Serializer`].
    output: &'a mut String,

    /// The Quadlet option currently being serialized.
    key: &'static str,

    /// Whether this Quadlet option sequence should be joined together with spaces into one value.
    join_option: bool,

    /// Whether the next value in the sequence is the first.
    ///
    /// Determines if "key=" or " " is written to the `output` before the value when `join_option`
    /// is `true`.
    first: bool,
}

impl<'a> SeqValueSerializer<'a> {
    /// Create a new [`SeqValueSerializer`].
    fn new(output: &'a mut String, key: &'static str, join_options: &HashSet<JoinOption>) -> Self {
        Self {
            output,
            key,
            join_option: key.parse().is_ok_and(|key| join_options.contains(&key)),
            first: true,
        }
    }

    /// Write a value to the `output`.
    fn write_value(&mut self, value: impl Display) {
        if self.join_option {
            if self.first {
                self.first = false;
                self.output.push_str(self.key);
                self.output.push('=');
            } else {
                self.output.push(' ');
            }
            // newline written in `<Self as ser::SerializeSeq>::end()`
            write!(self.output, "{value}").expect("writing to String never fails");
        } else {
            self.first = false;
            writeln!(self.output, "{}={value}", self.key).expect("writing to String never fails");
        }
    }
}

impl ser::SerializeSeq for SeqValueSerializer<'_> {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.join_option && !self.first {
            self.output.push('\n');
        }
        Ok(())
    }
}

impl ser::SerializeTuple for SeqValueSerializer<'_> {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for SeqValueSerializer<'_> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleVariant for SeqValueSerializer<'_> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::Serializer for &mut SeqValueSerializer<'_> {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Impossible<(), Error>;

    type SerializeTuple = Impossible<(), Error>;

    type SerializeTupleStruct = Impossible<(), Error>;

    type SerializeTupleVariant = Impossible<(), Error>;

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

    fn collect_str<T: ?Sized + Display>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        self.write_value(value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn basic_struct() -> Result<(), Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "PascalCase")]
        struct Test {
            one: u8,
            two: &'static str,
        }

        let sut = Test { one: 1, two: "two" };
        assert_eq!(
            to_string_join_all(sut)?,
            "[Test]\n\
            One=1\n\
            Two=two\n"
        );
        Ok(())
    }

    #[test]
    fn top_level_sequence() -> Result<(), Error> {
        #[derive(Serialize, Clone, Copy)]
        #[serde(rename_all = "PascalCase")]
        struct Test {
            one: u8,
        }

        let test = Test { one: 1 };
        let vec = vec![test, test];
        assert_eq!(
            to_string_join_all(vec)?,
            "[Test]\n\
            One=1\n\
            \n\
            [Test]\n\
            One=1\n"
        );
        Ok(())
    }

    #[test]
    fn top_level_tuple() -> Result<(), Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "PascalCase")]
        struct One {
            one: u8,
        }

        #[derive(Serialize)]
        #[serde(rename_all = "PascalCase")]
        struct Two {
            two: u8,
        }

        let tuple = (One { one: 1 }, Two { two: 2 });
        assert_eq!(
            to_string_join_all(tuple)?,
            "[One]\n\
            One=1\n\
            Two=2\n"
        );
        Ok(())
    }

    #[test]
    fn sequence() -> Result<(), Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "PascalCase")]
        struct Test {
            vec: Vec<u8>,
        }

        let sut = Test { vec: vec![1, 2, 3] };
        assert_eq!(
            to_string_join_all(sut)?,
            "[Test]\n\
            Vec=1\n\
            Vec=2\n\
            Vec=3\n"
        );
        Ok(())
    }

    #[test]
    fn sequence_join() -> Result<(), Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "PascalCase")]
        struct Test {
            #[serde(serialize_with = "seq_quote_whitespace")]
            after: Vec<&'static str>,
        }

        let sut = Test {
            after: vec!["one", "two three", "four"],
        };
        assert_eq!(
            to_string_join_all(sut)?,
            "[Test]\n\
            After=one \"two three\" four\n"
        );
        Ok(())
    }

    #[test]
    fn empty_is_empty() -> Result<(), Error> {
        #[derive(Serialize, Default)]
        #[serde(rename_all = "PascalCase")]
        struct Test {
            option: Option<&'static str>,
            vec: Vec<&'static str>,
            #[serde(serialize_with = "seq_quote_whitespace")]
            vec_joined: Vec<&'static str>,
        }

        assert_eq!(to_string_join_all(Test::default())?, "[Test]\n");
        Ok(())
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
        assert_eq!(to_string_join_all(sut), Err(Error::InvalidType));
    }

    #[test]
    fn quote_whitespace() {
        let quoted = QuoteWhitespace("test1 test2");
        assert_eq!(quoted.to_string(), r#""test1 test2""#);

        let quoted = QuoteWhitespace("test1\ntest2");
        assert_eq!(quoted.to_string(), r#""test1\ntest2""#);
    }
}
