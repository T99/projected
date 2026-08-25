use heck::ToUpperCamelCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

/// Resolved input for one generated ordering-enum variant.
pub struct OrderField {
	/// Conditional attributes copied from the source field.
	pub attrs: TokenStream,
	/// Normalized Rust field name used to construct the variant identifier.
	pub rust_name: String,
	/// Serde name emitted during serialization.
	pub serialize_name: String,
	/// Serde name accepted during deserialization.
	pub deserialize_name: String,
}

/// Complete input for one model's ordering API generation.
pub struct OrderInput {
	/// Resolved path to the `projected` runtime crate.
	pub runtime: TokenStream,
	/// Visibility inherited from the annotated source model.
	pub visibility: TokenStream,
	/// Model or projection type receiving `Orderable`.
	pub model_type: TokenStream,
	/// Identifier used to derive the `{Type}OrderField` name.
	pub model_ident: Ident,
	/// Generated shared field enum used as the mapping target.
	pub field_ident: Ident,
	/// Generic parameters for the generated implementation.
	pub impl_generics: TokenStream,
	/// Where clause preserved from the source model.
	pub where_clause: TokenStream,
	/// Orderable fields in source declaration order.
	pub fields: Vec<OrderField>,
}

impl OrderInput {
	/// Emits the Serde-facing order enum and its mapping to shared field metadata.
	///
	/// Serialization and deserialization names are emitted independently when
	/// directional Serde rules differ. No source attributes are reparsed here.
	pub fn emit(self) -> TokenStream {
		let OrderInput {
			runtime,
			visibility,
			model_type,
			model_ident,
			field_ident,
			impl_generics,
			where_clause,
			fields,
		} = self;
		let ordering_ident = crate::projected::names::order_field_type(&model_ident);
		let variants = fields.iter().map(|field| {
			let attrs = &field.attrs;
			let variant = format_ident!("{}", field.rust_name.to_upper_camel_case());
			let docs = format!("Orders by the `{}` field.", field.rust_name);
			let serialize_name = &field.serialize_name;
			let deserialize_name = &field.deserialize_name;
			if serialize_name == deserialize_name {
				quote! {
					#attrs
					#[doc = #docs]
					#[serde(rename = #serialize_name)]
					#variant
				}
			} else {
				quote! {
					#attrs
					#[doc = #docs]
					#[serde(rename(serialize = #serialize_name, deserialize = #deserialize_name))]
					#variant
				}
			}
		});
		let mapping = fields.iter().map(|field| {
			let attrs = &field.attrs;
			let variant = format_ident!("{}", field.rust_name.to_upper_camel_case());
			quote!(#attrs #ordering_ident::#variant => #field_ident::#variant,)
		});

		quote! {
			/// Serializable ordering fields generated for this model.
			#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
			#visibility enum #ordering_ident {
				#(#variants,)*
			}

			impl #impl_generics #runtime::Orderable for #model_type #where_clause {
				type OrderingField = #ordering_ident;

				fn projected_field(field: Self::OrderingField) -> Self::Field {
					match field {
						#(#mapping)*
					}
				}
			}
		}
	}
}
