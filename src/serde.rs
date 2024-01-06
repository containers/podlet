//! Provides [`serde::Serializer`]s for serializing command line args and quadlet files,
//! accessible through [`args::to_string()`] and [`quadlet::to_string()`].

/// Implement [`serde::Serializer`]'s `serialize_*` functions by returning `Err($error)`.
macro_rules! invalid_primitives {
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

pub mod args;
pub mod quadlet;

/// Skip serializing `bool`s that are `true`.
/// For use with `#[serde(skip_serializing_if = "skip_true")]`.
// ref required for serde's skip_serializing_if
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn skip_true(bool: &bool) -> bool {
    *bool
}
