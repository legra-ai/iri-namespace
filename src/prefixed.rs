//! [`PrefixedNamespace<T>`] — namespace + prefix, with syntax-aware
//! declaration and CURIE rendering.

use borrow_or_share::Bos;

use crate::namespace::Namespace;
use crate::prefix::Prefix;
use crate::syntax::{CurieSyntax, PrefixSyntax};

/// A [`Namespace`] paired with a [`Prefix`] — the chokepoint for
/// syntax-aware declaration and CURIE rendering.
///
/// Every parser, serializer, and prefix expander that used to
/// format namespace declarations or CURIEs ad hoc should go through
/// [`Self::declaration`] and [`Self::curie`] instead.
///
/// `Debug` and `Clone` are implemented manually under `T: Bos<str>`
/// because [`Namespace<T>`] only exposes those impls under that
/// bound.
pub struct PrefixedNamespace<T> {
    namespace: Namespace<T>,
    prefix: Prefix<T>,
}

impl<T: Bos<str> + core::fmt::Debug> core::fmt::Debug for PrefixedNamespace<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PrefixedNamespace")
            .field("namespace", &self.namespace)
            .field("prefix", &self.prefix)
            .finish()
    }
}

impl<T: Bos<str> + Clone> Clone for PrefixedNamespace<T> {
    fn clone(&self) -> Self {
        Self {
            namespace: self.namespace.clone(),
            prefix: self.prefix.clone(),
        }
    }
}

impl<T> PrefixedNamespace<T> {
    /// Pair a namespace with a prefix.
    ///
    /// Both inputs are pre-validated by their respective
    /// constructors — `Self::new` cannot fail.
    #[must_use]
    pub fn new(namespace: Namespace<T>, prefix: Prefix<T>) -> Self {
        Self { namespace, prefix }
    }

    /// The bound namespace.
    #[must_use]
    pub fn namespace(&self) -> &Namespace<T> {
        &self.namespace
    }

    /// The bound prefix.
    #[must_use]
    pub fn prefix(&self) -> &Prefix<T> {
        &self.prefix
    }
}

impl<T: Bos<str> + AsRef<str>> PrefixedNamespace<T> {
    /// Render the prefix-declaration block for `syntax`.
    ///
    /// Examples (with `prefix = "ex"` and
    /// `namespace.as_str() = "http://example.org/"`):
    ///
    /// | Syntax    | Output                                  |
    /// | --------- | --------------------------------------- |
    /// | `Sparql`  | `PREFIX ex: <http://example.org/>`      |
    /// | `Turtle`  | `@prefix ex: <http://example.org/> .`   |
    /// | `TriG`    | `@prefix ex: <http://example.org/> .`   |
    /// | `JsonLd`  | `"ex": "http://example.org/"`           |
    /// | `RdfXml`  | `xmlns:ex="http://example.org/"`        |
    #[must_use]
    pub fn declaration(&self, syntax: PrefixSyntax) -> String {
        syntax.format_declaration(self.prefix.as_str(), self.namespace.as_str())
    }

    /// Render a CURIE for `local` in `syntax`.
    ///
    /// `local` is appended verbatim — callers are expected to pass
    /// a syntactically-valid local-name suffix. Line syntaxes
    /// (`NTriples`, `NQuads`) ignore `prefix` and emit the full
    /// IRI in angle brackets.
    #[must_use]
    pub fn curie(&self, local: &str, syntax: CurieSyntax) -> String {
        if syntax.supports_curies() {
            format!("{}:{local}", self.prefix.as_str())
        } else {
            format!("<{}{local}>", self.namespace.as_str())
        }
    }
}
