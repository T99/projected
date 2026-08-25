use proc_macro2::Ident;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::{Token, parenthesized};

/// Parses a non-empty parenthesized, comma-separated identifier list.
///
/// `expected` names the list element in the empty-list diagnostic, allowing
/// projection fields and projection targets to share identical parsing rules.
pub fn parse_ident_list(input: ParseStream<'_>, expected: &str) -> syn::Result<Vec<Ident>> {
	let content;
	parenthesized!(content in input);
	let fields = Punctuated::<Ident, Token![,]>::parse_terminated(&content)?;
	if fields.is_empty() {
		return Err(syn::Error::new(
			content.span(),
			format!("expected at least one {expected}"),
		));
	}
	Ok(fields.into_iter().collect())
}
