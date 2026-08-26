use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::token;
use syn::punctuated::Punctuated;
use crate::util::parenthesized;

pub struct OptionalList<T: Parse> {
	pub span: Span,
	pub items: Option<Vec<T>>,
}

pub enum OptionalListParseError {
	EmptyList(Span),
	Malformed(Span),
}

pub fn parse_optional_list<T: Parse>(
	input: ParseStream,
) -> Result<OptionalList<T>, OptionalListParseError> {
	let span = input.span();
	if input.is_empty() || input.peek(token::Comma) {
		return Ok(OptionalList::<T> { span, items: None });
	} else if !input.peek(token::Paren) {
		return Err(OptionalListParseError::Malformed(span));
	}
	let content = parenthesized(input)
		.map_err(|err| OptionalListParseError::Malformed(err.span()))?;
	let items = Punctuated::<T, token::Comma>::parse_terminated(&content)
		.map_err(|err| OptionalListParseError::Malformed(err.span()))?;
	if items.is_empty() {
		return Err(OptionalListParseError::EmptyList(span));
	}
	Ok(OptionalList::<T> { span, items: Some(items.into_iter().collect()) })
}
