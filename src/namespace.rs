//! [`Namespace<T>`] — IRI + role with the suffix invariant pinned
//! at construction.

use borrow_or_share::Bos;
use iri_rfc3987::Iri;
use serde::{Deserialize, Serialize};

use crate::error::NamespaceError;
use crate::kind::NamespaceKind;

/// A namespace: an IRI whose string form ends in `#`, `/`, or — for
/// URN-scheme IRIs — `:`, paired with the role it plays in an RDF
/// dataset.
///
/// The suffix invariant means concatenating any local-name suffix
/// produces a syntactically valid IRI without re-parsing — the
/// per-call-site `if !iri.ends_with('#') { iri.push('#') }` checks
/// that historically caused the `…#workflow-instance##` bug are
/// unrepresentable.
///
/// **URN namespaces.** URN-scheme IRIs (RFC 8141) use `:` as the
/// namespace-specific-string separator instead of `#` or `/` — see
/// `urn:example:datatype:Branch`, `urn:isbn:0451450523`. The suffix
/// invariant therefore additionally accepts a trailing `:` when the
/// IRI's scheme is `urn`. Non-URN IRIs with a trailing `:` are still
/// rejected — `http://example.org:` is not a meaningful namespace.
///
/// `T` is the storage parameter mirroring [`Iri<T>`]:
/// - `Iri<&'static str>` for well-known namespace constants;
/// - `Iri<&'a str>` for a borrowed view;
/// - `Iri<String>` for runtime-synthesised namespaces.
///
/// `Debug` and `Clone` are implemented manually under `T: Bos<str>`
/// because `Iri<T>` only exposes those impls under that bound.
pub struct Namespace<T> {
    iri: Iri<T>,
    kind: NamespaceKind,
}

impl<T: Bos<str>> core::fmt::Debug for Namespace<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Namespace")
            .field("iri", &self.iri)
            .field("kind", &self.kind)
            .finish()
    }
}

impl<T: Bos<str> + Clone> Clone for Namespace<T> {
    fn clone(&self) -> Self {
        Self {
            iri: self.iri.clone(),
            kind: self.kind,
        }
    }
}

impl<T: Bos<str>, U: Bos<str>> PartialEq<Namespace<U>> for Namespace<T> {
    fn eq(&self, other: &Namespace<U>) -> bool {
        self.kind == other.kind && self.iri.as_str() == other.iri.as_str()
    }
}

impl<T: Bos<str>> Eq for Namespace<T> {}

impl<T: Bos<str>> core::hash::Hash for Namespace<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.iri.as_str().hash(state);
        self.kind.hash(state);
    }
}

impl<T: Bos<str>> Namespace<T> {
    /// Build a namespace from an IRI whose string form already ends
    /// in `#` or `/` (HTTP-style) or `:` (URN-style).
    ///
    /// # Errors
    ///
    /// Returns [`NamespaceError::MissingSuffix`] if the IRI does not
    /// end in one of the accepted separators. Use
    /// [`Namespace::from_iri_normalising`] when the input is a bare
    /// ontology IRI that needs `#` appended.
    pub fn try_new(iri: Iri<T>, kind: NamespaceKind) -> Result<Self, NamespaceError> {
        let s = iri.as_str();
        if has_namespace_suffix(s) {
            Ok(Self { iri, kind })
        } else {
            Err(NamespaceError::MissingSuffix { iri: s.to_owned() })
        }
    }

    /// The underlying IRI.
    #[must_use]
    pub fn iri(&self) -> &Iri<T> {
        &self.iri
    }

    /// The IRI as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.iri.as_str()
    }

    /// The classification.
    #[must_use]
    pub fn kind(&self) -> NamespaceKind {
        self.kind
    }

    /// Whether this namespace contributes terminology (T-box).
    ///
    /// Convenience wrapper over [`NamespaceKind::is_t_box`].
    #[must_use]
    pub fn is_t_box(&self) -> bool {
        self.kind.is_t_box()
    }

    /// Whether this namespace contributes instances (A-box).
    ///
    /// Convenience wrapper over [`NamespaceKind::is_a_box`].
    #[must_use]
    pub fn is_a_box(&self) -> bool {
        self.kind.is_a_box()
    }

    /// Borrow this namespace as `Namespace<&str>` — zero-copy view
    /// over the underlying IRI bytes.
    ///
    /// # Panics
    ///
    /// Panics only if the underlying IRI string fails to re-parse —
    /// impossible because the `Iri<T>` was already validated at
    /// construction time. The `expect` is documentation of that
    /// invariant.
    #[must_use]
    pub fn as_borrowed(&self) -> Namespace<&str> {
        Namespace {
            iri: Iri::<&str>::parse(self.as_str())
                .expect("constructed namespaces hold a valid IRI"),
            kind: self.kind,
        }
    }

    /// Return the local-name suffix for `term` if it is in this
    /// namespace, or `None` otherwise.
    ///
    /// `try_abbreviate(&iri)` returns `Some("Foo")` when
    /// `iri.as_str()` starts with `self.as_str()` and the suffix is
    /// non-empty; otherwise `None`. The non-empty check rules out a
    /// CURIE-empty case where the local name would collapse the IRI
    /// to the namespace itself.
    #[must_use]
    pub fn try_abbreviate<'a, U: Bos<str>>(&self, term: &'a Iri<U>) -> Option<&'a str> {
        let prefix = self.as_str();
        let full = term.as_str();
        let local = full.strip_prefix(prefix)?;
        if local.is_empty() { None } else { Some(local) }
    }
}

impl Namespace<&'static str> {
    /// Build a namespace from a known-valid static IRI string.
    ///
    /// The ergonomic door for compile-time-known namespace constants
    /// (e.g. `RDF`, `RDFS`, `OWL`). Pairs naturally with
    /// `std::sync::LazyLock` for the well-known namespace table.
    ///
    /// # Panics
    ///
    /// Panics if `iri` is not a valid IRI per RFC 3987, or if it
    /// does not end in `#`, `/`, or a URN namespace `:`. Both checks catch the failure at
    /// first reference to the `LazyLock` rather than at runtime use.
    #[must_use]
    pub fn known(iri: &'static str, kind: NamespaceKind) -> Self {
        let parsed = Iri::<&'static str>::parse(iri)
            .unwrap_or_else(|e| panic!("Namespace::known called with invalid IRI {iri:?}: {e}"));
        Self::try_new(parsed, kind)
            .unwrap_or_else(|e| panic!("Namespace::known called with invalid namespace: {e}"))
    }
}

impl Namespace<String> {
    /// Build a namespace from a bare IRI, appending `#` when the
    /// IRI does not already end in `#` or `/`.
    ///
    /// The sanctioned door for "I got this IRI from an
    /// `owl:Ontology` declaration; normalise it for use as a CURIE
    /// prefix". The historical scanner bug —
    /// `…workflow-instance#` becoming `…workflow-instance##` —
    /// cannot happen here.
    ///
    /// # Panics
    ///
    /// Panics only if appending `#` to an already-valid IRI fails
    /// to re-parse — impossible by RFC 3987 (a `#` introduces an
    /// empty fragment, valid in any IRI). The `expect` is
    /// documentation of that invariant.
    #[must_use]
    pub fn from_iri_normalising(iri: Iri<String>, kind: NamespaceKind) -> Self {
        if has_namespace_suffix(iri.as_str()) {
            Self { iri, kind }
        } else {
            let mut s = iri.into_string();
            s.push('#');
            let normalised = Iri::<String>::parse_owned(s)
                .expect("appending '#' to a valid IRI yields a syntactically valid IRI");
            Self {
                iri: normalised,
                kind,
            }
        }
    }
}

fn has_namespace_suffix(s: &str) -> bool {
    s.ends_with('#') || s.ends_with('/') || is_urn_namespace_suffix(s)
}

/// URN-scheme namespaces (RFC 8141) end in `:` — the namespace-
/// specific-string separator. Restricted to the `urn:` scheme so a
/// stray trailing colon on an HTTP IRI is still rejected.
fn is_urn_namespace_suffix(s: &str) -> bool {
    s.starts_with("urn:") && s.ends_with(':')
}

/// Wire form: a namespace travels across crate, persistence, and
/// network boundaries as `{ iri, kind }`. The IRI string carries
/// the suffix invariant on the wire; deserialization re-checks it
/// through [`Namespace::try_new`], so a malformed payload is
/// rejected at deserialize time rather than at later use.
#[derive(Serialize, Deserialize)]
struct NamespaceWire {
    iri: String,
    kind: NamespaceKind,
}

impl<T: Bos<str>> Serialize for Namespace<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        NamespaceWire {
            iri: self.as_str().to_owned(),
            kind: self.kind,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Namespace<String> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = NamespaceWire::deserialize(deserializer)?;
        let iri = Iri::<String>::parse_owned(wire.iri)
            .map_err(|(e, _)| serde::de::Error::custom(format!("invalid namespace IRI: {e}")))?;
        Namespace::try_new(iri, wire.kind)
            .map_err(|e| serde::de::Error::custom(format!("invalid namespace: {e}")))
    }
}
