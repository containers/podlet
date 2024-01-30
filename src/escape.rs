//! Utilities for escaping strings.

use std::borrow::Cow;

/// Join an iterator of command arguments into a [`String`], [quoting](arg_quote()) when necessary.
///
/// Each argument is separated by a space.
pub(crate) fn command_join<I>(args: I) -> String
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let mut args = args.into_iter();

    let (lower, upper) = args.size_hint();
    let mut string = String::with_capacity(upper.unwrap_or(lower) * 2);

    if let Some(first) = args.next() {
        string.push_str(&arg_quote(first.as_ref()));
    }

    for arg in args {
        string.push(' ');
        string.push_str(&arg_quote(arg.as_ref()));
    }

    string
}

/// Encode a string for use as a shell argument.
///
/// ASCII control characters that are not whitespace are silently removed.
pub(crate) fn arg_quote(arg: &str) -> Cow<str> {
    if arg.contains(char_is_ascii_control_not_whitespace) {
        let arg = arg.replace(char_is_ascii_control_not_whitespace, "");
        shlex::try_quote(&arg)
            .expect("null characters have been removed")
            .into_owned()
            .into()
    } else {
        shlex::try_quote(arg).expect("string does not contain null character")
    }
}

/// Checks if the character is an ASCII control character and is not an ASCII whitespace character.
fn char_is_ascii_control_not_whitespace(char: char) -> bool {
    // Do not match on "Horizontal Tab" (\t, \x09), "Line Feed" (\n, \x0A), "Vertical Tab" (\x0B),
    // "Form Feed" (\x0C), or "Carriage Return" (\r, \x0D).
    char.is_ascii_control() && !matches!(char, '\x09'..='\x0D')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_remove_control() {
        assert_eq!(arg_quote("te\0st"), "test");
        assert_eq!(arg_quote("hello\nworld"), "'hello\nworld'");
    }

    #[test]
    fn join() {
        assert_eq!(command_join(["test", "hello world"]), "test 'hello world'");
    }
}
