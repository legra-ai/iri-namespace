# iri-namespace

[![Crates.io](https://img.shields.io/crates/v/iri-namespace.svg)](https://crates.io/crates/iri-namespace)
[![Downloads](https://img.shields.io/crates/d/iri-namespace.svg)](https://crates.io/crates/iri-namespace)
[![Documentation](https://docs.rs/iri-namespace/badge.svg)](https://docs.rs/iri-namespace)
[![CI](https://github.com/legra-ai/iri-namespace/actions/workflows/ci.yml/badge.svg)](https://github.com/legra-ai/iri-namespace/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/legra-ai/iri-namespace)

Validated namespace, prefix, and CURIE types for RDF and related syntaxes.

`iri-namespace` provides the small type layer between an IRI parser and
applications that need to declare prefixes or abbreviate terms. It does not
contain a vocabulary registry: callers provide their own namespace IRIs and
prefix names.

## Why this crate

Namespace handling is easy to make stringly typed:

- a namespace without a trailing `#`, `/`, or URN `:` cannot safely receive a
  local name;
- a prefix containing `:` is not a valid XML `NCName`;
- each RDF-adjacent syntax has a different prefix declaration spelling;
- owned values are often copied even when a borrowed view would be enough.

This crate makes those invariants explicit while keeping storage generic:

- `Namespace<T>` stores an RFC 3987 IRI and a namespace role;
- `Prefix<T>` validates an XML `NCName`;
- `PrefixedNamespace<T>` combines the two and renders declarations or CURIEs;
- `Namespace<&str>` and `Prefix<&str>` provide zero-copy borrowed views;
- `Namespace<String>` supports runtime-owned values.

## Quick start

```rust
use iri_namespace::{
    CurieSyntax,
    Namespace,
    NamespaceKind,
    Prefix,
    PrefixSyntax,
    PrefixedNamespace,
};

let namespace = Namespace::known(
    "https://example.org/vocab#",
    NamespaceKind::Ontology,
);
let prefix = Prefix::known("ex");
let binding = PrefixedNamespace::new(namespace, prefix);

assert_eq!(
    binding.declaration(PrefixSyntax::Sparql),
    "PREFIX ex: <https://example.org/vocab#>",
);
assert_eq!(
    binding.curie("Person", CurieSyntax::Turtle),
    "ex:Person",
);
```

`Namespace::known` and `Prefix::known` are intended for compile-time-known
values. They fail immediately if a constant is invalid. Runtime values use
the fallible constructors:

```rust
use iri_namespace::{Namespace, NamespaceKind, Prefix};
use iri_rfc3987::Iri;

let namespace = Namespace::try_new(
    Iri::<String>::known("https://example.org/runtime/"),
    NamespaceKind::Resource,
)?;
let prefix = Prefix::try_new("runtime".to_owned())?;

assert_eq!(namespace.as_str(), "https://example.org/runtime/");
assert_eq!(prefix.as_str(), "runtime");
# Ok::<(), iri_namespace::NamespaceError>(())
```

## Namespace invariants

`Namespace::try_new` accepts HTTP-style namespace IRIs ending in `#` or `/`,
and URN-scheme namespace IRIs ending in `:`. A bare IRI can be normalized with
`Namespace::from_iri_normalising`, which appends `#` exactly once.

`NamespaceKind` records whether a namespace represents an ontology, shape,
datatype, or resource role. Its `is_t_box` and `is_a_box` helpers expose the
coarse terminology-versus-instance partition without requiring a registry or
global mutable state.

## Prefix and CURIE rendering

`PrefixSyntax` renders one declaration in the syntax selected by the caller:

| Syntax | Output for `ex` and `https://example.org/vocab#` |
| --- | --- |
| `Sparql` | `PREFIX ex: <https://example.org/vocab#>` |
| `Turtle` / `TriG` | `@prefix ex: <https://example.org/vocab#> .` |
| `JsonLd` | `"ex": "https://example.org/vocab#"` |
| `RdfXml` | `xmlns:ex="https://example.org/vocab#"` |

`CurieSyntax::NTriples` and `CurieSyntax::NQuads` deliberately emit a full
IRI in angle brackets because those line syntaxes do not support prefix
declarations.

## Scope

This crate validates and formats namespace values. It does not provide:

- RDF, RDFS, OWL, XSD, or application-specific namespace constants;
- a namespace registry or prefix-resolution environment;
- RDF parsing, serialization, storage, or network I/O;
- vocabulary semantics beyond the namespace role carried by `NamespaceKind`.

## License

Licensed under either of:

- MIT license ([`LICENSE-MIT`](LICENSE-MIT) or <https://opensource.org/licenses/MIT>);
- Apache License, Version 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>).

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in this crate by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
