//! [`Prefix<T>`] — XML-`NCName`-validated prefix name.

use crate::error::NamespaceError;

/// A prefix name (the part before `:` in a CURIE), validated as an
/// XML `NCName`.
///
/// `NCName` production (simplified to ASCII for this crate):
/// - first character: ASCII letter (`a`–`z`, `A`–`Z`) or `_`;
/// - subsequent characters: ASCII letter, ASCII digit, `-`, `.`, or `_`;
/// - no `:` anywhere.
///
/// Empty strings and strings containing `:` are rejected. The
/// "prefix has a `:` in it" class of bugs is unrepresentable.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Prefix<T>(T);

impl<T: AsRef<str>> Prefix<T> {
    /// Validate `value` as an `NCName` and wrap it.
    ///
    /// # Errors
    ///
    /// Returns [`NamespaceError::NotNcName`] when the value is
    /// empty, contains a `:`, starts with a non-letter/non-`_`
    /// character, or contains a character outside
    /// `[A-Za-z0-9_.-]`.
    pub fn try_new(value: T) -> Result<Self, NamespaceError> {
        if is_ncname(value.as_ref()) {
            Ok(Self(value))
        } else {
            Err(NamespaceError::NotNcName {
                prefix: value.as_ref().to_owned(),
            })
        }
    }

    /// The prefix name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    /// Borrow this prefix as `Prefix<&str>`.
    #[must_use]
    pub fn as_borrowed(&self) -> Prefix<&str> {
        Prefix(self.as_str())
    }
}

impl Prefix<&'static str> {
    /// Wrap a known-valid static prefix string.
    ///
    /// The ergonomic door for compile-time-known prefix constants
    /// (e.g. `"rdf"`, `"owl"`). Pairs naturally with
    /// `std::sync::LazyLock` for the well-known prefix table.
    ///
    /// # Panics
    ///
    /// Panics if `value` is not a valid XML `NCName`. The check
    /// fires at first reference to the `LazyLock`.
    #[must_use]
    pub fn known(value: &'static str) -> Self {
        Self::try_new(value)
            .unwrap_or_else(|e| panic!("Prefix::known called with invalid NCName: {e}"))
    }
}

fn is_ncname(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !is_ncname_start(first) {
        return false;
    }
    chars.all(is_ncname_continue)
}

fn is_ncname_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ncname_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')
}
