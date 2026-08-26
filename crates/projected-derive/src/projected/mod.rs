use crate::projected::models::{AttributePolicy, SourceModel};
use crate::util::{fields_mut, real_crate_path};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, DeriveInput, Fields, ItemStruct, Meta, parse_macro_input, parse_quote};

/// Conditional and propagated attribute handling.
mod attributes;
/// Backend-neutral metadata and ordering emitters.
mod codegen;
/// Parsed and resolved compile-time model.
mod models;
/// Collision-safe names for generated support items.
mod names;
/// Directional Serde name resolution.
mod serde_name;

/// Parses the public attribute target and installs the hidden expansion phase.
///
/// Errors are converted to compiler diagnostics at the original source span.
pub fn projected(args: TokenStream, input: TokenStream) -> TokenStream {
	let args = proc_macro2::TokenStream::from(args);
	let mut item = parse_macro_input!(input as ItemStruct);
	match prepare_item(args, &mut item) {
		Ok(()) => quote!(#item).into(),
		Err(error) => error.into_compile_error().into(),
	}
}

/// Runs the hidden derive phase against the struct representation visible after
/// all preceding transforming attributes have expanded.
pub fn derive_projected(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	match expand(input) {
		Ok(tokens) => tokens.into(),
		Err(error) => error.into_compile_error().into(),
	}
}

/// Captures safe source derives and rewrites public field controls into inert
/// helper attributes consumed by `__Projected`.
///
/// The hidden derive is appended so a following `sea_orm::model` attribute can
/// copy it onto SeaORM's generated `Model` and `ModelEx` representations.
fn prepare_item(args: proc_macro2::TokenStream, item: &mut ItemStruct) -> syn::Result<()> {
	let safe_derives = AttributePolicy::safe_derives(&item.attrs)?;
	for field in fields_mut(&mut item.fields) {
		for attr in &mut field.attrs {
			if attr.path().is_ident("projected") {
				*attr = internal_attribute(attr)?;
			}
		}
	}

	let runtime = real_crate_path("projected")?;
	item.attrs
		.push(parse_quote!(#[derive(#runtime::__Projected)]));
	if args.is_empty() {
		item.attrs.push(parse_quote!(#[projected_internal]));
	} else {
		item.attrs.push(parse_quote!(#[projected_internal(#args)]));
	}
	if !safe_derives.is_empty() {
		item.attrs
			.push(parse_quote!(#[projected_internal(projection_derives(#(#safe_derives),*))]));
	}
	Ok(())
}



/// Converts a public field-level `#[projected(...)]` attribute into hidden
/// metadata while preserving its original argument tokens and spans.
fn internal_attribute(attr: &Attribute) -> syn::Result<Attribute> {
	match &attr.meta {
		Meta::List(list) => {
			let tokens = &list.tokens;
			Ok(parse_quote!(#[projected_internal(#tokens)]))
		}
		Meta::Path(_) | Meta::NameValue(_) => Err(syn::Error::new_spanned(
			attr,
			"expected #[projected(...)] on a field",
		)),
	}
}

/// Parses, resolves, and emits one post-transformation model.
///
/// SeaORM's generated relationship-bearing companion is intentionally
/// suppressed here to prevent duplicate projection modules and support types.
fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
	let source = SourceModel::parse(input)?;
	if source.sea_orm.is_generated_companion {
		return Ok(proc_macro2::TokenStream::new());
	}
	let resolved = source.resolve()?;
	resolved.emit()
}
