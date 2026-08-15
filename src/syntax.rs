//! Syntaxes for [`PrefixedNamespace`](crate::PrefixedNamespace)
//! declaration and CURIE rendering.

/// Concrete-syntax flavour for a prefix-declaration block.
///
/// Used by
/// [`PrefixedNamespace::declaration`](crate::PrefixedNamespace::declaration).
/// Each variant corresponds to the canonical way that syntax
/// introduces a namespace binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrefixSyntax {
    /// `PREFIX ex: <http://example.org/>`
    Sparql,
    /// `@prefix ex: <http://example.org/> .`
    Turtle,
    /// `@prefix ex: <http://example.org/> .` (`TriG` shares
    /// Turtle's declaration shape).
    TriG,
    /// `"ex": "http://example.org/"` (one entry inside a JSON-LD
    /// `@context` object — caller is responsible for the
    /// surrounding `{ … }`).
    JsonLd,
    /// `xmlns:ex="http://example.org/"`
    RdfXml,
}

/// Concrete-syntax flavour for a CURIE (compact IRI) rendering.
///
/// Used by [`PrefixedNamespace::curie`](crate::PrefixedNamespace::curie).
/// Compact syntaxes (SPARQL, Turtle, Cypher, JSON-LD) emit the
/// `prefix:local` form; line-oriented N-Triples / N-Quads do not
/// support prefixes and must emit the full IRI in angle brackets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CurieSyntax {
    /// `ex:Foo`
    Sparql,
    /// `ex:Foo`
    Turtle,
    /// `ex:Foo` (`TriG` shares Turtle's CURIE shape).
    TriG,
    /// `ex:Foo` (Cypher accepts CURIEs in `n10s:`-extended mode).
    Cypher,
    /// `ex:Foo` inside `@context`-bound JSON-LD term references.
    JsonLd,
    /// `<http://example.org/Foo>` — line-syntax fallback for
    /// N-Triples (no prefix declarations exist).
    NTriples,
    /// `<http://example.org/Foo>` — line-syntax fallback for
    /// N-Quads (no prefix declarations exist).
    NQuads,
}

impl CurieSyntax {
    /// Whether this syntax supports prefix-based abbreviation.
    ///
    /// `false` for line syntaxes (`NTriples`, `NQuads`) — they must
    /// emit the full IRI even when a prefix is in scope upstream.
    #[must_use]
    pub const fn supports_curies(self) -> bool {
        !matches!(self, Self::NTriples | Self::NQuads)
    }
}

impl PrefixSyntax {
    /// Format a single prefix-declaration line for this syntax,
    /// given a raw `prefix` label and namespace `iri`.
    ///
    /// This is the chokepoint that
    /// [`PrefixedNamespace::declaration`](crate::PrefixedNamespace::declaration)
    /// dispatches through, exposed as a free helper for runtime
    /// callers that hold a parsed prefix label and IRI string
    /// without going through
    /// [`Prefix::try_new`](crate::Prefix::try_new) /
    /// [`Namespace::try_new`](crate::Namespace::try_new) validation
    /// — typical of algebra display, SPARQL prologue rendering, and
    /// CRM query builders where the (label, IRI) pair already came
    /// out of a validated upstream parser or compile-time constant.
    ///
    /// Output examples for `prefix = "ex"`,
    /// `iri = "http://example.org/"`:
    ///
    /// | Syntax    | Output                                  |
    /// | --------- | --------------------------------------- |
    /// | `Sparql`  | `PREFIX ex: <http://example.org/>`      |
    /// | `Turtle`  | `@prefix ex: <http://example.org/> .`   |
    /// | `TriG`    | `@prefix ex: <http://example.org/> .`   |
    /// | `JsonLd`  | `"ex": "http://example.org/"`           |
    /// | `RdfXml`  | `xmlns:ex="http://example.org/"`        |
    #[must_use]
    pub fn format_declaration(self, prefix: &str, iri: &str) -> String {
        match self {
            Self::Sparql => format!("PREFIX {prefix}: <{iri}>"),
            Self::Turtle | Self::TriG => format!("@prefix {prefix}: <{iri}> ."),
            Self::JsonLd => format!("\"{prefix}\": \"{iri}\""),
            Self::RdfXml => format!("xmlns:{prefix}=\"{iri}\""),
        }
    }
}
