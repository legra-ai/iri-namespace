//! Namespace IRIs preserve the allocation visible to caller accounting.

use iri_namespace::{Namespace, NamespaceKind};
use iri_rfc3987::Iri;

#[test]
fn namespace_exposes_its_owned_iri_capacity() {
    let mut text = String::with_capacity(4096);
    text.push_str("urn:namespace:allocated:");
    let capacity = text.capacity();
    let namespace =
        Namespace::try_new(Iri::parse_owned(text).unwrap(), NamespaceKind::Ontology).unwrap();
    let iri: &Iri<String> = namespace.iri();
    assert_eq!(iri.capacity(), capacity);
}
