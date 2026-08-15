#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod error;
mod kind;
mod namespace;
mod prefix;
mod prefixed;
mod syntax;

#[cfg(test)]
mod tests;

pub use error::NamespaceError;
pub use kind::NamespaceKind;
pub use namespace::Namespace;
pub use prefix::Prefix;
pub use prefixed::PrefixedNamespace;
pub use syntax::{CurieSyntax, PrefixSyntax};
