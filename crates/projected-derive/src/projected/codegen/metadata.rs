use proc_macro2::{Ident, TokenStream};
use quote::quote;

/// Resolved metadata needed to emit one generated field-enum variant.
pub struct MetadataField {
	/// Conditional attributes copied from the source field.
	pub attrs: TokenStream,
	/// Variant emitted for the current model or projection.
	pub variant: Ident,
	/// Corresponding variant on the source model's field enum.
	pub source_variant: Ident,
	/// Normalized Rust field name stored in runtime metadata.
	pub rust_name: String,
	/// Resolved serialization name stored in runtime metadata.
	pub serialize_name: String,
	/// Resolved deserialization name stored in runtime metadata.
	pub deserialize_name: String,
	/// Whether ordering generation may expose the field.
	pub orderable: bool,
}

impl MetadataField {
	/// Emits this field's enum variant with synchronized conditional attributes
	/// and downstream-facing documentation.
	pub fn emit_variant(&self) -> TokenStream {
		let attrs = &self.attrs;
		let variant = &self.variant;
		let docs = format!("Logical field identity for `{}`.", self.rust_name);
		quote! {
			#attrs
			#[doc = #docs]
			#variant
		}
	}

	/// Emits one match arm returning the field's normalized runtime metadata.
	pub fn emit_metadata_arm(&self, runtime: &TokenStream) -> TokenStream {
		let attrs = &self.attrs;
		let variant = &self.variant;
		let rust_name = &self.rust_name;
		let serialize_name = &self.serialize_name;
		let deserialize_name = &self.deserialize_name;
		let orderable = self.orderable;
		quote! {
			#attrs
			Self::#variant => #runtime::FieldMetadata::new(
				#rust_name,
				#serialize_name,
				#deserialize_name,
				#orderable,
			),
		}
	}

	/// Emits one value in the model's source-ordered static field slice.
	pub fn emit_field_value(&self, field_ident: &Ident) -> TokenStream {
		let attrs = &self.attrs;
		let variant = &self.variant;
		quote!(#attrs #field_ident::#variant,)
	}

	/// Emits one projection-field-to-source-field mapping arm.
	pub fn emit_source_arm(&self, source_field_ident: &Ident) -> TokenStream {
		let attrs = &self.attrs;
		let variant = &self.variant;
		let source_variant = &self.source_variant;
		quote!(#attrs Self::#variant => #source_field_ident::#source_variant,)
	}
}

/// Complete input for backend-neutral runtime metadata generation.
pub struct MetadataInput {
	/// Resolved path to the `projected` runtime crate.
	pub runtime: TokenStream,
	/// Visibility inherited from the annotated source model.
	pub visibility: TokenStream,
	/// Model or projection type receiving `ProjectedModel`.
	pub model_type: TokenStream,
	/// Generated field enum identifier for `model_type`.
	pub field_ident: Ident,
	/// Original source model type used by field mapping.
	pub source_type: TokenStream,
	/// Original source model's generated field enum identifier.
	pub source_field_ident: Ident,
	/// Generic parameters for generated implementation blocks.
	pub impl_generics: TokenStream,
	/// Where clause preserved from the source model.
	pub where_clause: TokenStream,
	/// Fields represented by this model, in source declaration order.
	pub fields: Vec<MetadataField>,
}

impl MetadataInput {
	/// Emits the field enum and implementations of `ProjectedField`,
	/// `ProjectedModel`, and `ProjectedFieldMapping`.
	///
	/// The same emitter serves source models and projections. For a source model,
	/// the mapping is identity; for a projection, every selected variant maps to
	/// its original source variant.
	pub fn emit(self) -> TokenStream {
		let MetadataInput {
			runtime,
			visibility,
			model_type,
			field_ident,
			source_type,
			source_field_ident,
			impl_generics,
			where_clause,
			fields,
		} = self;
		let variants = fields.iter().map(MetadataField::emit_variant);
		let metadata_arms = fields.iter().map(|field| field.emit_metadata_arm(&runtime));
		let field_values = fields
			.iter()
			.map(|field| field.emit_field_value(&field_ident));
		let source_arms = fields
			.iter()
			.map(|field| field.emit_source_arm(&source_field_ident));

		quote! {
			/// Backend-neutral logical fields generated for this model.
			#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
			#visibility enum #field_ident {
				#(#variants,)*
			}

			impl #runtime::ProjectedField for #field_ident {
				fn metadata(self) -> #runtime::FieldMetadata {
					match self {
						#(#metadata_arms)*
					}
				}
			}

			impl #impl_generics #runtime::ProjectedModel for #model_type #where_clause {
				type Field = #field_ident;

				fn fields() -> &'static [Self::Field] {
					&[
						#(#field_values)*
					]
				}
			}

			impl #impl_generics #runtime::ProjectedFieldMapping<#source_type> for #field_ident #where_clause {
				fn source_field(self) -> <#source_type as #runtime::ProjectedModel>::Field {
					match self {
						#(#source_arms)*
					}
				}
			}
		}
	}
}
