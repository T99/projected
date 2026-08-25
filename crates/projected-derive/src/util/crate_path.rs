/// Resolves a dependency name as it is visible from the consuming crate.
///
/// `proc-macro-crate` accounts for Cargo dependency aliases. When expansion is
/// occurring inside the named crate itself, this returns `crate`; otherwise it
/// returns an absolute path. A missing direct dependency becomes a diagnostic
/// at the macro call site rather than an unresolved generated path.
pub fn real_crate_path(crate_name: &str) -> syn::Result<proc_macro2::TokenStream> {
	match proc_macro_crate::crate_name(crate_name) {
		Ok(proc_macro_crate::FoundCrate::Itself) => Ok(quote::quote!(crate)),
		Ok(proc_macro_crate::FoundCrate::Name(alias)) => {
			let ident = syn::Ident::new(&alias, proc_macro2::Span::call_site());
			Ok(quote::quote!(::#ident))
		}
		Err(_) => Err(syn::Error::new(
			proc_macro2::Span::call_site(),
			format!(
				"could not find crate `{crate_name}`; add it to the consuming crate's dependencies"
			),
		)),
	}
}
