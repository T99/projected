use proc_macro2::Ident;
use syn::{Attribute, Data, DeriveInput, Field, Fields, Generics, Visibility};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use crate::projected::source_model_field::SourceModelField;

pub struct SourceModel {
	/// Source type identifier visible to the hidden derive phase.
	pub ident: Ident,
	/// Visibility copied to generated public support types.
	pub visibility: Visibility,
	/// Lifetimes, type parameters, const parameters, and where clause.
	pub generics: Generics,
	/// Source attributes retained for propagation and backend detection.
	pub attrs: Vec<Attribute>,
	/// Named fields in declaration order with shared resolved metadata.
	pub fields: Vec<SourceModelField>,
	/// Struct-level projection declarations in source order.
	pub declarations: Vec<ProjectionDeclaration>,
	/// Module placement and source ordering options.
	pub options: ProjectedOptions,
	/// Derive inheritance and attribute propagation policy.
	pub attributes: AttributePolicy,
}

impl SourceModel {
	
	/// Parse a `DeriveInput` into a `SourceModel`.
	///
	/// # Args
	/// * `input` - The input to parse.
	///
	/// # Returns
	/// A `syn::Result` containing the parsed `SourceModel` or an error if
	/// parsing fails.
	pub fn parse(input: DeriveInput) -> syn::Result<Self> {
		let fields = Self::parse_raw_fields(input.data, &input.ident)?;
		for attr in &input.attrs {
		
		}
		Ok(Self {
			ident: input.ident,
			visibility: input.vis,
			generics: input.generics,
			attrs: input.attrs,
		})
	}
	
	/// Parse the raw fields from the input data, ensuring that it is a struct
	/// with named fields.
	///
	/// # Args
	/// * `data` - The input data to parse.
	/// * `ident` - The identifier of the struct, used for error reporting.
	///
	/// # Returns
	/// A `syn::Result` containing the parsed fields or an error if the input
	/// data is not a struct with named fields.
	fn parse_raw_fields(
		data: Data,
		ident: &Ident,
	) -> syn::Result<Punctuated<Field, Comma>> {
		let Data::Struct(struct_data) = data else {
			return Err(syn::Error::new_spanned(
				ident,
				"projected can only be used on structs with named fields",
			));
		};
		let Fields::Named(fields) = struct_data.fields else {
			return Err(syn::Error::new_spanned(
				ident,
				"projected can only be used on structs with named fields; tuple and unit structs are not supported",
			));
		};
		Ok(fields.named)
	}
}