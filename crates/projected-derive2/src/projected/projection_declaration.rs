use proc_macro2::{Ident, Span};
use syn::parse::{Parse, ParseStream};
use crate::projected::options::ProjectionDeclarationOption;
use crate::util::{parse_optional_list, OptionalListParseError};

pub struct ProjectionDeclaration {
	pub span: Span,
	pub ident: Ident,
	pub options: Option<Vec<ProjectionDeclarationOption>>,
}

impl Parse for ProjectionDeclaration {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let span = input.span();
		let ident: Ident = input.parse()?;
		let options = parse_optional_list(&input)
			.map_err(|err| match err {
				OptionalListParseError::EmptyList(span) => syn::Error::new(
					span,
					"expected at least one projection declaration option (omit the parentheses to apply no options) e.g. `include(field1, field2)` or `exclude`"
				),
				OptionalListParseError::Malformed(span) => syn::Error::new(
					span,
					"expected a parenthesized list of projection declaration options (or nothing to apply no options) e.g. `include(field1, field2)` or `exclude`"
				),
			})?
			.items
			.and_then(|items| items.into());
		Ok(Self {
			span,
			ident,
			options
		})
	}
}