use proc_macro2::{Ident, Span};
use syn::parse::{Parse, ParseStream};
use crate::util::{parse_optional_list, OptionalListParseError};

/// An enumeration of the possible options available when declaring a projection
/// in a struct marked as `#[projected]`.
///
/// # Examples
/// ```rust,ignore
/// #[projected(
///   projections(
///     projection1(include(field1, field2), exclude(field3)),
///     projection2(include(field4), optional(field5)),
///   ),
/// )]
/// struct MyStruct { /* ... */ }
/// ```
pub enum ProjectionDeclarationOption {
	Include {
		span: Span,
		fields: Option<Vec<Ident>>,
	},
	Exclude {
		span: Span,
		fields: Option<Vec<Ident>>,
	},
	Optional {
		span: Span,
		fields: Option<Vec<Ident>>,
	},
}

impl ProjectionDeclarationOption {
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
	fn parse_field_name_list(input: ParseStream) -> syn::Result<Option<Vec<Ident>>> {
		// FIXME - Currently this method will accept bare 'include' and
		//   'exclude', which it should not do.
		Ok(parse_optional_list(&input)
			.map_err(|err| match err {
				OptionalListParseError::EmptyList(span) => syn::Error::new(
					span,
					"expected at least one field name (omit the \
					parentheses to include all fields) e.g. `exclude(\
					field1, field2)` or `include`"
				),
				OptionalListParseError::Malformed(span) => syn::Error::new(
					span,
					"expected a parenthesized list of field names (or nothing \
					to apply to all fields) e.g. `include(field1, \
					field2)` or `exclude`"
				),
			})?
			.items
			.and_then(|items| items.into()))
	}
}

impl Parse for ProjectionDeclarationOption {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let span = input.span();
		let name: Ident = input.parse()?;
		if name == "include" {
			Ok(Self::Include {
				span,
				fields: Self::parse_field_name_list(input)?,
			})
		} else if name == "exclude" {
			Ok(Self::Exclude {
				span,
				fields: Self::parse_field_name_list(input)?,
			})
		} else if name == "optional" {
			Ok(Self::Optional {
				span,
				fields: Self::parse_field_name_list(input)?,
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
