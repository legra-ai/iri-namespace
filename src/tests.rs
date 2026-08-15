//! Unit tests covering every invariant and rendering path.

use iri_rfc3987::Iri;

use crate::{
    CurieSyntax, Namespace, NamespaceError, NamespaceKind, Prefix, PrefixSyntax, PrefixedNamespace,
};

// ── Namespace::try_new ──────────────────────────────────────────────

#[test]
fn try_new_accepts_hash_terminated_iri() {
    let iri = Iri::<String>::known("http://example.org/ontology#");
    let ns = Namespace::try_new(iri, NamespaceKind::Ontology).expect("hash suffix");
    assert_eq!(ns.as_str(), "http://example.org/ontology#");
    assert_eq!(ns.kind(), NamespaceKind::Ontology);
}

#[test]
fn try_new_accepts_slash_terminated_iri() {
    let iri = Iri::<String>::known("http://example.org/resources/");
    let ns = Namespace::try_new(iri, NamespaceKind::Resource).expect("slash suffix");
    assert_eq!(ns.as_str(), "http://example.org/resources/");
}

#[test]
fn try_new_rejects_bare_iri() {
    let iri = Iri::<String>::known("http://example.org/bare");
    let err = Namespace::try_new(iri, NamespaceKind::Ontology).unwrap_err();
    assert!(matches!(err, NamespaceError::MissingSuffix { .. }));
}

#[test]
fn try_new_accepts_urn_colon_terminated_iri() {
    let iri = Iri::<String>::known("urn:example:datatype:");
    let ns = Namespace::try_new(iri, NamespaceKind::Datatype).expect("urn colon suffix");
    assert_eq!(ns.as_str(), "urn:example:datatype:");
    assert_eq!(ns.kind(), NamespaceKind::Datatype);
}

#[test]
fn try_new_rejects_non_urn_colon_terminated_iri() {
    // A trailing `:` on an HTTP scheme is not a meaningful namespace
    // separator — the URN suffix rule applies only to the `urn:` scheme.
    let iri = Iri::<String>::known("http://example.org:");
    let err = Namespace::try_new(iri, NamespaceKind::Ontology).unwrap_err();
    assert!(matches!(err, NamespaceError::MissingSuffix { .. }));
}

#[test]
fn try_abbreviate_works_for_urn_namespace() {
    let ns = Namespace::known("urn:example:datatype:", NamespaceKind::Datatype);
    let term = Iri::<String>::known("urn:example:datatype:Branch");
    assert_eq!(ns.try_abbreviate(&term), Some("Branch"));
}

// ── Namespace::known (static-table ergonomics) ──────────────────────

#[test]
fn known_accepts_valid_static_namespace() {
    let ns = Namespace::known("http://www.w3.org/2002/07/owl#", NamespaceKind::Ontology);
    assert_eq!(ns.as_str(), "http://www.w3.org/2002/07/owl#");
    assert!(ns.is_t_box());
}

#[test]
#[should_panic(expected = "must end in")]
fn known_panics_on_missing_suffix() {
    let _ = Namespace::known("http://www.w3.org/2002/07/owl", NamespaceKind::Ontology);
}

#[test]
#[should_panic(expected = "invalid IRI")]
fn known_panics_on_unparseable_iri() {
    let _ = Namespace::known("not an iri", NamespaceKind::Ontology);
}

// ── Namespace::from_iri_normalising ─────────────────────────────────

#[test]
fn from_iri_normalising_appends_hash_to_bare_iri() {
    let iri = Iri::<String>::known("http://example.org/ontology");
    let ns = Namespace::from_iri_normalising(iri, NamespaceKind::Ontology);
    assert_eq!(ns.as_str(), "http://example.org/ontology#");
}

#[test]
fn from_iri_normalising_does_not_double_hash() {
    let iri = Iri::<String>::known("http://example.org/workflow-instance#");
    let ns = Namespace::from_iri_normalising(iri, NamespaceKind::Ontology);
    assert_eq!(
        ns.as_str(),
        "http://example.org/workflow-instance#",
        "must not produce '…workflow-instance##' — the historical scanner bug"
    );
}

#[test]
fn from_iri_normalising_leaves_slash_suffix_alone() {
    let iri = Iri::<String>::known("http://example.org/resources/");
    let ns = Namespace::from_iri_normalising(iri, NamespaceKind::Resource);
    assert_eq!(ns.as_str(), "http://example.org/resources/");
}

// ── NamespaceKind T-box / A-box partition ───────────────────────────

#[test]
fn ontology_shape_datatype_are_t_box() {
    assert!(NamespaceKind::Ontology.is_t_box());
    assert!(NamespaceKind::Shape.is_t_box());
    assert!(NamespaceKind::Datatype.is_t_box());
    assert!(!NamespaceKind::Ontology.is_a_box());
}

#[test]
fn resource_is_a_box() {
    assert!(NamespaceKind::Resource.is_a_box());
    assert!(!NamespaceKind::Resource.is_t_box());
}

#[test]
fn namespace_carries_partition_views() {
    let iri = Iri::<String>::known("http://example.org/types/");
    let ns = Namespace::try_new(iri, NamespaceKind::Datatype).unwrap();
    assert!(ns.is_t_box());
    assert!(!ns.is_a_box());
}

// ── Namespace::as_borrowed ──────────────────────────────────────────

#[test]
fn as_borrowed_preserves_kind_and_string_form() {
    let iri = Iri::<String>::known("http://example.org/ontology#");
    let owned = Namespace::try_new(iri, NamespaceKind::Ontology).unwrap();
    let borrowed = owned.as_borrowed();
    assert_eq!(borrowed.as_str(), owned.as_str());
    assert_eq!(borrowed.kind(), owned.kind());
}

// ── Namespace::try_abbreviate ───────────────────────────────────────

#[test]
fn try_abbreviate_returns_local_suffix_in_namespace() {
    let ns = Namespace::try_new(
        Iri::<String>::known("http://example.org/ontology#"),
        NamespaceKind::Ontology,
    )
    .unwrap();
    let term = Iri::<String>::known("http://example.org/ontology#Foo");
    assert_eq!(ns.try_abbreviate(&term), Some("Foo"));
}

#[test]
fn try_abbreviate_returns_none_for_foreign_namespace() {
    let ns = Namespace::try_new(
        Iri::<String>::known("http://example.org/ontology#"),
        NamespaceKind::Ontology,
    )
    .unwrap();
    let term = Iri::<String>::known("http://other.example.org/ontology#Foo");
    assert_eq!(ns.try_abbreviate(&term), None);
}

#[test]
fn try_abbreviate_returns_none_for_empty_local_name() {
    let ns = Namespace::try_new(
        Iri::<String>::known("http://example.org/ontology#"),
        NamespaceKind::Ontology,
    )
    .unwrap();
    let same_as_namespace = Iri::<String>::known("http://example.org/ontology#");
    assert_eq!(
        ns.try_abbreviate(&same_as_namespace),
        None,
        "empty suffix would collapse the CURIE to the bare prefix"
    );
}

// ── Prefix::try_new (NCName validation) ─────────────────────────────

#[test]
fn prefix_accepts_simple_letters_and_digits() {
    assert!(Prefix::try_new("ex").is_ok());
    assert!(Prefix::try_new("rdfs").is_ok());
    assert!(Prefix::try_new("a1").is_ok());
    assert!(Prefix::try_new("_under").is_ok());
}

#[test]
fn prefix_accepts_internal_punctuation() {
    assert!(Prefix::try_new("ex-1").is_ok());
    assert!(Prefix::try_new("ex.1").is_ok());
    assert!(Prefix::try_new("ex_1").is_ok());
}

#[test]
fn prefix_rejects_empty_string() {
    let err = Prefix::try_new("").unwrap_err();
    assert!(matches!(err, NamespaceError::NotNcName { .. }));
}

#[test]
fn prefix_rejects_digit_or_punctuation_start() {
    assert!(Prefix::try_new("1ex").is_err());
    assert!(Prefix::try_new("-ex").is_err());
    assert!(Prefix::try_new(".ex").is_err());
}

#[test]
fn prefix_rejects_colon_anywhere() {
    let err = Prefix::try_new("ex:bad").unwrap_err();
    assert!(matches!(err, NamespaceError::NotNcName { .. }));
}

#[test]
fn prefix_rejects_whitespace_and_specials() {
    assert!(Prefix::try_new("ex ample").is_err());
    assert!(Prefix::try_new("ex/").is_err());
    assert!(Prefix::try_new("ex#").is_err());
}

#[test]
fn prefix_known_accepts_valid_static_prefix() {
    let p = Prefix::known("rdfs");
    assert_eq!(p.as_str(), "rdfs");
}

#[test]
#[should_panic(expected = "invalid NCName")]
fn prefix_known_panics_on_invalid_ncname() {
    let _ = Prefix::known("1bad");
}

// ── Serde round-trip ─────────────────────────────────────────────────

#[test]
fn namespace_serde_round_trip() {
    let ns = Namespace::known("http://example.org/onto#", NamespaceKind::Ontology);
    let json = serde_json::to_string(&ns).expect("serialize");
    assert!(json.contains("\"iri\":\"http://example.org/onto#\""));
    assert!(json.contains("\"kind\":\"Ontology\""));
    let decoded: Namespace<String> = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decoded.as_str(), ns.as_str());
    assert_eq!(decoded.kind(), ns.kind());
}

#[test]
fn namespace_deserialize_rejects_missing_suffix() {
    let json = r#"{"iri":"http://example.org/bare","kind":"Ontology"}"#;
    let err = serde_json::from_str::<Namespace<String>>(json).unwrap_err();
    assert!(
        err.to_string().contains("must end in"),
        "expected suffix error, got: {err}"
    );
}

#[test]
fn namespace_deserialize_rejects_invalid_iri() {
    let json = r#"{"iri":"not an iri","kind":"Ontology"}"#;
    let err = serde_json::from_str::<Namespace<String>>(json).unwrap_err();
    assert!(
        err.to_string().contains("invalid namespace IRI"),
        "expected IRI parse error, got: {err}"
    );
}

#[test]
fn namespace_kind_serializes_as_pascal_case() {
    // PascalCase matches the wire format and its discriminator strings.
    assert_eq!(
        serde_json::to_string(&NamespaceKind::Ontology).unwrap(),
        "\"Ontology\""
    );
    assert_eq!(
        serde_json::to_string(&NamespaceKind::Shape).unwrap(),
        "\"Shape\""
    );
    assert_eq!(
        serde_json::to_string(&NamespaceKind::Datatype).unwrap(),
        "\"Datatype\""
    );
    assert_eq!(
        serde_json::to_string(&NamespaceKind::Resource).unwrap(),
        "\"Resource\""
    );
}

// ── PrefixedNamespace::declaration ──────────────────────────────────

fn example_prefixed() -> PrefixedNamespace<String> {
    let ns = Namespace::try_new(
        Iri::<String>::known("http://example.org/"),
        NamespaceKind::Resource,
    )
    .unwrap();
    let prefix = Prefix::try_new("ex".to_owned()).unwrap();
    PrefixedNamespace::new(ns, prefix)
}

#[test]
fn declaration_sparql() {
    let pn = example_prefixed();
    assert_eq!(
        pn.declaration(PrefixSyntax::Sparql),
        "PREFIX ex: <http://example.org/>"
    );
}

#[test]
fn declaration_turtle() {
    let pn = example_prefixed();
    assert_eq!(
        pn.declaration(PrefixSyntax::Turtle),
        "@prefix ex: <http://example.org/> ."
    );
}

#[test]
fn declaration_trig_matches_turtle() {
    let pn = example_prefixed();
    assert_eq!(
        pn.declaration(PrefixSyntax::TriG),
        pn.declaration(PrefixSyntax::Turtle)
    );
}

#[test]
fn declaration_jsonld_entry_form() {
    let pn = example_prefixed();
    assert_eq!(
        pn.declaration(PrefixSyntax::JsonLd),
        "\"ex\": \"http://example.org/\""
    );
}

#[test]
fn declaration_rdfxml_xmlns_form() {
    let pn = example_prefixed();
    assert_eq!(
        pn.declaration(PrefixSyntax::RdfXml),
        "xmlns:ex=\"http://example.org/\""
    );
}

// ── PrefixedNamespace::curie ────────────────────────────────────────

#[test]
fn curie_compact_syntaxes_use_prefix_local_form() {
    let pn = example_prefixed();
    for syntax in [
        CurieSyntax::Sparql,
        CurieSyntax::Turtle,
        CurieSyntax::TriG,
        CurieSyntax::Cypher,
        CurieSyntax::JsonLd,
    ] {
        assert_eq!(pn.curie("Foo", syntax), "ex:Foo", "{syntax:?}");
    }
}

#[test]
fn curie_line_syntaxes_emit_full_iri_in_brackets() {
    let pn = example_prefixed();
    for syntax in [CurieSyntax::NTriples, CurieSyntax::NQuads] {
        assert_eq!(
            pn.curie("Foo", syntax),
            "<http://example.org/Foo>",
            "{syntax:?}",
        );
    }
}

#[test]
fn supports_curies_partitions_syntaxes_correctly() {
    assert!(CurieSyntax::Sparql.supports_curies());
    assert!(CurieSyntax::Turtle.supports_curies());
    assert!(CurieSyntax::TriG.supports_curies());
    assert!(CurieSyntax::Cypher.supports_curies());
    assert!(CurieSyntax::JsonLd.supports_curies());
    assert!(!CurieSyntax::NTriples.supports_curies());
    assert!(!CurieSyntax::NQuads.supports_curies());
}
