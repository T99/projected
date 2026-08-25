/// A generated partial representation that can be completed into its base value.
pub trait Projection {
	/// The named-field struct from which this projection was generated.
	type Base;

	/// Values required to reconstruct [`Self::Base`].
	///
	/// This is `()` for a lossless projection. Otherwise it is a generated
	/// missing-values struct containing excluded values and fallbacks for
	/// projection-optional fields.
	type Missing;

	/// Reconstructs the base value from projected and missing values.
	///
	/// For an optional projection field, an outer `Some` wins and its contained
	/// value is used verbatim. An outer `None` selects the corresponding fallback
	/// from `missing`; this preserves the distinction between omission and an
	/// explicit null represented by `Some(None)`.
	fn complete(self, missing: Self::Missing) -> Self::Base;
}
