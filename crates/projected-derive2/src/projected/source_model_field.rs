use proc_macro2::Ident;
use syn::{Attribute, Type};

pub struct SourceModelField {
	/// Rust field identifier.
	pub ident: Ident,
	/// Rust field type exactly as declared after transforming attributes.
	pub ty: Type,
	/// Original field attributes retained for controlled propagation and `cfg`.
	pub attrs: Vec<Attribute>,
	/// Directional Serde-visible names.
	pub names: FieldNames,
	/// Whether ordering may expose this field.
	pub orderable: bool,
	/// Projection-selection rules declared directly on this field.
	pub rules: Vec<FieldRule>,
}