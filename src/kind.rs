//! Classification of a namespace's role in an RDF dataset.

/// The role a [`Namespace`](crate::Namespace) plays in an RDF
/// dataset.
///
/// Captured here (rather than discarded after stats computation as
/// the previous registry did) so the kind can be persisted on
/// registry entries and carried across wire boundaries. The four
/// variants compose into a coarse T-box / A-box partition via
/// [`Namespace::is_t_box`](crate::Namespace::is_t_box) and
/// [`Namespace::is_a_box`](crate::Namespace::is_a_box).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum NamespaceKind {
    /// Terms defined by an `owl:Ontology` declaration.
    ///
    /// Classifies as T-box.
    Ontology,
    /// Terms defined by a SHACL shapes graph.
    ///
    /// Classifies as T-box.
    Shape,
    /// Datatype namespace (e.g. `xsd:`, `geo:`).
    ///
    /// Classifies as T-box.
    Datatype,
    /// Instance namespace — owns A-box individuals.
    Resource,
}

impl NamespaceKind {
    /// Whether this kind contributes terminology (classes,
    /// properties, datatypes, shapes) rather than instances.
    #[must_use]
    pub const fn is_t_box(self) -> bool {
        matches!(self, Self::Ontology | Self::Shape | Self::Datatype)
    }

    /// Whether this kind contributes individuals rather than
    /// terminology.
    #[must_use]
    pub const fn is_a_box(self) -> bool {
        matches!(self, Self::Resource)
    }
}
