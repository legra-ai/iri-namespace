//! Construction errors for [`Namespace`](crate::Namespace) and
//! [`Prefix`](crate::Prefix).

/// Construction failure for a [`Namespace`](crate::Namespace) or
/// [`Prefix`](crate::Prefix).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamespaceError {
    /// The IRI does not end in an accepted namespace separator. A
    /// namespace IRI must be usable as the prefix portion of a
    /// CURIE — concatenating a local name yields a syntactically
    /// valid IRI only when the namespace already ends in `#` or
    /// `/` (HTTP-style) or `:` (URN-scheme IRIs per RFC 8141, e.g.
    /// `urn:example:datatype:`).
    MissingSuffix {
        /// The offending IRI string.
        iri: String,
    },
    /// The string is not a valid XML `NCName`.
    ///
    /// `NCName`: first character is a letter (`a`–`z`, `A`–`Z`) or
    /// `_`; subsequent characters are letters, digits, `-`, `.`,
    /// or `_`. Empty strings and strings containing `:` are
    /// rejected.
    NotNcName {
        /// The offending prefix string.
        prefix: String,
    },
}

impl core::fmt::Display for NamespaceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingSuffix { iri } => write!(
                f,
                "namespace IRI must end in '#', '/', or ':' (for urn:-scheme IRIs), got {iri:?}",
            ),
            Self::NotNcName { prefix } => write!(f, "prefix is not a valid XML NCName: {prefix:?}"),
        }
    }
}

impl core::error::Error for NamespaceError {}
