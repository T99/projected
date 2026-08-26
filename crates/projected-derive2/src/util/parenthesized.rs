use syn::parse::ParseBuffer;

pub fn parenthesized(
	input: syn::parse::ParseStream,
) -> syn::Result<ParseBuffer> {
	let content;
	syn::parenthesized!(content in input);
	Ok(content)
}