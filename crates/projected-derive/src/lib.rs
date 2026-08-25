//! Attribute-macro implementation for the `projected` runtime crate.
//!
//! The public attribute performs orchestration and records configuration for a
//! hidden derive phase. Keeping both phases here allows expansion to run after
//! transforming attributes such as `sea_orm::model` without exposing helper
//! macros to downstream users.

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use proc_macro::TokenStream;

/// Public and hidden macro-phase orchestration.
mod projected;
/// Shared syntax and crate-resolution utilities.
mod util;

/// Generates backend-neutral field metadata, owned projections, and shared
/// query-field APIs for a named-field struct.
///
/// Struct configuration declares projections, optional module placement,
/// projection derives/attribute propagation, and source ordering. Field
/// configuration controls per-projection inclusion/optionality and ordering
/// eligibility. The same `#[projected(...)]` name is used at both levels.
///
/// Place this attribute before transforming attributes such as
/// `#[sea_orm::model]`. It installs a hidden derive that runs against the
/// transformed representation, allowing dense SeaORM models to generate from
/// scalar `Model` exactly once while suppressing duplicate `ModelEx` output.
///
/// See the `projected` runtime crate's public API guide for complete syntax and
/// examples.
#[proc_macro_attribute]
pub fn projected(args: TokenStream, input: TokenStream) -> TokenStream {
	projected::projected(args, input)
}

/// Executes the internal, post-transformation expansion phase.
///
/// This derive is emitted by [`projected`] and is not a downstream-facing API.
#[doc(hidden)]
#[proc_macro_derive(__Projected, attributes(projected_internal, serde, sea_orm))]
pub fn derive_projected(input: TokenStream) -> TokenStream {
	projected::derive_projected(input)
}
