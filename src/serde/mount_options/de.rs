//! Provides [`Deserializer`] for [`from_str()`](super::from_str()).

use std::str::{FromStr, SplitTerminator};

use serde::{
    de::{self, DeserializeSeed, Visitor, value::BorrowedStrDeserializer},
    forward_to_deserialize_any,
};

use super::Error;

/// A deserializer for parsing strings as mount options.
///
/// Only supports deserializing to structs.
pub struct Deserializer<'de> {
    input: &'de str,
}

impl<'de> Deserializer<'de> {
    /// Create a [`Deserializer`] from an input string.
    pub fn from_str(input: &'de str) -> Self {
        Self { input }
    }
}

impl<'de> de::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
        enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // Err(Error::BadType)
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(MapAccess::from_str(self.input))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }
}

/// Deserializes maps for [`Deserializer`].
struct MapAccess<'de> {
    split: SplitTerminator<'de, char>,
    next_value: Option<&'de str>,
}

impl<'de> MapAccess<'de> {
    /// Create a [`MapAccess`] from an input string by splitting it with ','.
    fn from_str(input: &'de str) -> Self {
        Self {
            split: input.split_terminator(','),
            next_value: None,
        }
    }
}

impl<'de> de::MapAccess<'de> for MapAccess<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        let Some(next) = self.split.next() else {
            return Ok(None);
        };

        let (key, next_value) = next.split_once('=').map_or((next, None), |(key, value)| {
            (key, (!value.is_empty()).then_some(value))
        });
        self.next_value = next_value;

        seed.deserialize(BorrowedStrDeserializer::new(key))
            .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        if let Some(value) = self.next_value.take() {
            seed.deserialize(ValueDeserializer::from_str(value))
        } else {
            seed.deserialize(NoValueDeserializer)
        }
    }
}

/// Deserializes values for [`MapAccess`]/[`Deserializer`].
struct ValueDeserializer<'de> {
    input: &'de str,
}

impl<'de> ValueDeserializer<'de> {
    /// Create a [`ValueDeserializer`] from a string.
    fn from_str(input: &'de str) -> Self {
        Self { input }
    }
}

impl<'de> de::Deserializer<'de> for ValueDeserializer<'de> {
    type Error = Error;

    forward_to_deserialize_any! {
        bytes byte_buf unit unit_struct newtype_struct seq tuple tuple_struct
        map struct identifier ignored_any
    }

    deserialize_parse! {
        'de, input,
        deserialize_bool => visit_bool,
        deserialize_i8 => visit_i8,
        deserialize_i16 => visit_i16,
        deserialize_i32 => visit_i32,
        deserialize_i64 => visit_i64,
        deserialize_i128 => visit_i128,
        deserialize_u8 => visit_u8,
        deserialize_u16 => visit_u16,
        deserialize_u32 => visit_u32,
        deserialize_u64 => visit_u64,
        deserialize_u128 => visit_u128,
        deserialize_f32 => visit_f32,
        deserialize_f64 => visit_f64,
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let input = self.input;

        if input.is_empty() {
            visitor.visit_none()
        } else if input == "true" {
            visitor.visit_bool(true)
        } else if input == "false" {
            visitor.visit_bool(false)
        } else if let Ok(u64) = u64::from_str(input) {
            visitor.visit_u64(u64)
        } else if let Ok(i64) = i64::from_str(input) {
            visitor.visit_i64(i64)
        } else if let Ok(f64) = f64::from_str(input) {
            visitor.visit_f64(f64)
        } else if input.chars().count() == 1 {
            self.deserialize_char(visitor)
        } else {
            self.deserialize_str(visitor)
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut chars = self.input.chars();
        match chars.next() {
            Some(char) if chars.next().is_none() => visitor.visit_char(char),
            _ => Err(de::Error::invalid_type(
                de::Unexpected::Str(self.input),
                &visitor,
            )),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.input)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.input.to_owned())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(BorrowedStrDeserializer::new(self.input))
    }
}

/// Deserializer when no value is available.
///
/// Similar to [`UnitDeserializer`](de::value::UnitDeserializer) except that `bool`s are always
/// deserialized as `true`.
struct NoValueDeserializer;

impl<'de> de::Deserializer<'de> for NoValueDeserializer {
    type Error = Error;

    forward_to_deserialize_any! {
        i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(true)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }
}
