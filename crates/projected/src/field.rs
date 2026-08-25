/// Backend-neutral metadata for a generated logical model field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldMetadata {
	/// The field's Rust identifier without a raw-identifier prefix such as `r#`.
	pub rust_name: &'static str,
	/// The field name used when serializing, after applying Serde rename rules.
	pub serialize_name: &'static str,
	/// The field name accepted when deserializing, after applying Serde rename rules.
	pub deserialize_name: &'static str,
	/// Whether ordering generation may expose this field.
	pub orderable: bool,
}

impl FieldMetadata {
	/// Creates metadata for generated field-identity implementations.
	///
	/// This constructor is public only so code emitted into downstream crates can
	/// build metadata without exposing implementation details in the macro crate.
	#[doc(hidden)]
	pub const fn new(
		rust_name: &'static str,
		serialize_name: &'static str,
		deserialize_name: &'static str,
		orderable: bool,
	) -> Self {
		Self {
			rust_name,
			serialize_name,
			deserialize_name,
			orderable,
		}
	}
}

/// A generated backend-neutral field identity.
pub trait ProjectedField: Copy + Eq + std::fmt::Debug + 'static {
	/// Returns the names and query capabilities resolved for this logical field.
	///
	/// Serde-visible names have already been normalized by `#[projected]`; backend
	/// integrations should consume this value instead of reparsing attributes.
	fn metadata(self) -> FieldMetadata;
}

/// A model with generated logical field metadata.
pub trait ProjectedModel {
	/// The generated enum that identifies this model's logical fields.
	type Field: ProjectedField;

	/// Returns all logical fields in source declaration order.
	///
	/// Conditional fields retain their source `cfg` conditions, so this slice
	/// always agrees with the fields available in the current compilation.
	fn fields() -> &'static [Self::Field];
}

/// Maps a generated field identity back to a source model's field identity.
///
/// For a source model this is an identity mapping. A projection maps each of
/// its selected fields to the corresponding field on its base model.
pub trait ProjectedFieldMapping<Source: ProjectedModel>: ProjectedField {
	/// Returns the source-model identity represented by this field.
	///
	/// Source model field enums map to themselves. Projection field enums map
	/// only their selected fields and therefore provide a lossless link back to
	/// the shared source metadata.
	fn source_field(self) -> Source::Field;
}
