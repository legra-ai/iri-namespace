//! Exercise namespace construction and serialization through the public API.
use iri_namespace::{
    CurieSyntax, Namespace, NamespaceKind, Prefix, PrefixSyntax, PrefixedNamespace,
};

#[test]
fn validated_binding_renders_curie_and_full_iri() {
    let namespace = Namespace::known("https://example.org/vocab#", NamespaceKind::Ontology);
    let binding = PrefixedNamespace::new(namespace, Prefix::known("ex"));
    assert_eq!(
        binding.declaration(PrefixSyntax::Sparql),
        "PREFIX ex: <https://example.org/vocab#>"
    );
    assert_eq!(binding.curie("Person", CurieSyntax::Turtle), "ex:Person");
    assert_eq!(
        binding.curie("Person", CurieSyntax::NTriples),
        "<https://example.org/vocab#Person>"
    );
    let serialized = serde_json::to_string(binding.namespace()).expect("serialize namespace");
    let restored: Namespace<String> =
        serde_json::from_str(&serialized).expect("validated namespace");
    assert_eq!(restored.as_borrowed(), binding.namespace().as_borrowed());
}

#[test]
fn invalid_external_values_cannot_bypass_namespace_invariants() {
    assert!(Prefix::try_new("not:a:prefix").is_err());
    let external = r#"{"iri":"https://example.org/bare","kind":"Ontology"}"#;
    assert!(serde_json::from_str::<Namespace<String>>(external).is_err());
}
