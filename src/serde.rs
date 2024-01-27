//! Provides [`serde::Serializer`]s for serializing command line args and quadlet files,
//! accessible through [`args::to_string()`] and [`quadlet::to_string()`].
//!
//! Also provides a [`serde::Serializer`] and [`serde::Deserializer`] for (de)serializing mount
//! options via [`mount_options::to_string()`] and [`mount_options::from_str()`].

use std::fmt::Display;

use serde::{ser::SerializeSeq, Serializer};

/// Implement [`serde::Serializer`]'s `serialize_*` functions by returning `Err($error)`.
macro_rules! serialize_invalid_primitives {
    ($error:expr, $($f:ident: $t:ty,)*) => {
        $(
            fn $f(self, _v: $t) -> Result<Self::Ok, Self::Error> {
                Err($error)
            }
        )*
    };
}

/// Implement [`serde::Serializer`]'s `serialize_*` functions by executing `self.$write_fn(v)`.
macro_rules! serialize_primitives {
    ($write_fn:ident, $($f:ident: $t:ty,)*) => {
        $(
            fn $f(self, v: $t) -> Result<Self::Ok, Self::Error> {
                self.$write_fn(v);
                Ok(())
            }
        )*
    };
}

/// Implement [`serde::Deserializer`]'s `deserialize_*` functions by parsing `input` and calling the
/// appropriate [`serde::de::Visitor`] function.
macro_rules! deserialize_parse {
    ($de:lifetime, $input:ident, $($f:ident => $visit:ident,)*) => {
        $(
            fn $f<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: ::serde::de::Visitor<$de>,
            {
                if let Ok(value) = self.$input.parse() {
                    visitor.$visit(value)
                } else {
                    Err(::serde::de::Error::invalid_type(
                        ::serde::de::Unexpected::Str(self.$input),
                        &visitor,
                    ))
                }
            }
        )*
    };
}

pub mod args;
pub mod mount_options;
pub mod quadlet;

/// Skip serializing `bool`s that are `true`.
/// For use with `#[serde(skip_serializing_if = "skip_true")]`.
// ref required for serde's skip_serializing_if
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn skip_true(bool: &bool) -> bool {
    *bool
}

/// Skip serializing default values
pub fn skip_default<T>(value: &T) -> bool
where
    T: Default + PartialEq,
{
    *value == T::default()
}

/// Serialize a sequence of items as strings using their [`Display`] implementation.
pub fn serialize_display_seq<'a, T, S>(value: &'a T, serializer: S) -> Result<S::Ok, S::Error>
where
    &'a T: IntoIterator,
    <&'a T as IntoIterator>::Item: Display,
    S: Serializer,
{
    let iter = value.into_iter();
    let len = iter.size_hint().1;

    let mut state = serializer.serialize_seq(len)?;

    for item in iter {
        state.serialize_element(&item.to_string())?;
    }

    state.end()
}
