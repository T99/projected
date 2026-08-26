use proc_macro2::{Ident, Span};
use syn::meta::ParseNestedMeta;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use crate::util::{parse_optional_list, OptionalListParseError};

/// An enumeration of the possible options available when using the
/// `#[projected(...)]` attribute on the fields of a struct marked as
/// `#[projected]`.
///
/// # Examples
/// ```rust,ignore
/// struct MyStruct {
///   #[projected(
///     include(Projection1, Projection2),
///     exclude(Projection3),
///     optional(Projection4))
///   ]
///   my_field: String,
/// }
/// ```
pub enum ProjectedFieldOption {
	/// Includes the field in the specified projections.
	///
	/// If no projections are specified, the field is included in all projections.
	Include {
		/// The span over this option, used for diagnostics.
		span: Span,
		/// The projections for which the marked field is included.
		///
		/// If `None`, the field is included in all projections.
		projections: Option<Vec<Ident>>,
	},
	/// Excludes the field from the specified projections.
	///
	/// If no projections are specified, the field is excluded from all projections.
	Exclude {
		/// The span over this option, used for diagnostics.
		span: Span,
		/// The projections for which the marked field is excluded.
		///
		/// If `None`, the field is excluded from all projections.
		projections: Option<Vec<Ident>>,
	},
	/// Marks the field as optional in the specified projections.
	///
	/// If no projections are specified, the field is optional in all projections.
	Optional {
		/// The span over this option, used for diagnostics.
		span: Span,
		/// The projections for which the marked field is optional.
		///
		/// If `None`, the field is optional in all projections.
		projections: Option<Vec<Ident>>,
	},
}

impl ProjectedFieldOption {
	/// Parses a list of projection names from the input stream, returning an
	/// `Option<Vec<Ident>>`.
	///
	/// # Arguments
	/// * `input` - The input stream to parse from.
	///
	/// # Returns
	/// * `Ok(Some(Vec<Ident>))` if a list of projection names is successfully parsed.
	/// * `Ok(None)` if no list is present (indicating all projections).
	/// * `Err(syn::Error)` if the list is malformed or empty.
	fn parse_projection_name_list(input: ParseStream) -> syn::Result<Option<Vec<Ident>>> {
		Ok(parse_optional_list(&input)
			.map_err(|err| match err {
				OptionalListParseError::EmptyList(span) => syn::Error::new(
					span,
					"expected at least one projection name (omit the \
					parentheses to apply to all projections) e.g. `include(\
					projection1, projection2)` or `exclude`"
				),
				OptionalListParseError::Malformed(span) => syn::Error::new(
					span,
					"expected a parenthesized list of projection names (or nothing \
					to apply to all projections) e.g. `include(projection1, \
					projection2)` or `exclude`"
				),
			})?
			.items
			.and_then(|items| items.into()))
	}
}

impl Parse for ProjectedFieldOption {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let span = input.span();
		let name: Ident = input.parse()?;
		if name == "include" {
			Ok(Self::Include {
				span,
				projections: Self::parse_projection_name_list(input)?,
			})
		} else if name == "exclude" {
			Ok(Self::Exclude {
				span,
				projections: Self::parse_projection_name_list(input)?,
			})
		} else if name == "optional" {
			Ok(Self::Optional {
				span,
				projections: Self::parse_projection_name_list(input)?,
			})
		} else {
			Err(syn::Error::new(
				name.span(),
				format!("unknown projected option `{name}`; expected \
				 `include`, `exclude`, or `optional`"),
			))
		}
	}
}
