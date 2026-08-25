use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{Attribute, Meta, Token, parse_quote};

/// Extracts conditional attributes that must remain synchronized across every
/// generated item derived from a source field.
///
/// Plain `cfg` attributes are copied directly. A `cfg_attr` is retained only
/// when one of its nested attributes is itself `cfg`; unrelated nested
/// attributes are deliberately discarded here.
pub fn conditional(attrs: &[Attribute]) -> Vec<Attribute> {
	attrs
		.iter()
		.filter_map(|attr| {
			if attr.path().is_ident("cfg") {
				Some(attr.clone())
			} else if attr.path().is_ident("cfg_attr") {
				filtered_cfg_attr(attr, |meta| meta.path().is_ident("cfg"))
			} else {
				None
			}
		})
		.collect()
}

/// Returns whether conditional compilation may remove an item entirely.
///
/// Generic-usage analysis treats such a field conservatively because it cannot
/// rely on that field to carry a generic parameter in every configuration.
pub fn may_remove_item(attrs: &[Attribute]) -> bool {
	!conditional(attrs).is_empty()
}

/// Rebuilds a `cfg_attr` with only nested metadata accepted by `keep`.
///
/// The leading condition is always preserved. Malformed input or an empty
/// filtered body returns `None`, preventing invalid or meaningless generated
/// attributes from escaping the macro.
pub fn filtered_cfg_attr(attr: &Attribute, keep: impl Fn(&Meta) -> bool) -> Option<Attribute> {
	let Meta::List(list) = &attr.meta else {
		return None;
	};
	let mut arguments = Punctuated::<Meta, Token![,]>::parse_terminated
		.parse2(list.tokens.clone())
		.ok()?
		.into_iter();
	let condition = arguments.next()?;
	let nested = arguments.filter(keep).collect::<Vec<_>>();
	if nested.is_empty() {
		return None;
	}
	Some(parse_quote!(#[cfg_attr(#condition, #(#nested),*)]))
}
