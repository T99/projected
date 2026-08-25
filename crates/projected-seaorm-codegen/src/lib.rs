//! Internal SeaORM token generation used by the `projected` macro orchestrator.
//!
//! This crate is a regular Rust library rather than a proc-macro crate. It owns
//! only entity-specific `ActiveModel` conversion tokens; parsing, validation,
//! and generic projection generation remain in `projected-derive`.

#![warn(missing_docs)]

use proc_macro2::{Ident, TokenStream};
use quote::quote;

/// A selected projection field used to initialize one `ActiveModel` field.
pub struct FieldInput {
	/// Conditional attributes copied from the source field.
	pub attrs: TokenStream,
	/// The shared source, projection, and active-model field identifier.
	pub ident: Ident,
	/// Whether the projection adds an outer omission-aware `Option`.
	pub optional: bool,
}

impl FieldInput {
	/// Emits one `ActiveModel` struct initializer.
	///
	/// Required values are always `Set`. Projection-optional values distinguish
	/// omission (`None` becomes `NotSet`) from presence (`Some(value)` becomes
	/// `Set(value)`), preserving nested-option null semantics.
	pub fn emit_active_field(&self, sea_orm: &TokenStream) -> TokenStream {
		let attrs = &self.attrs;
		let ident = &self.ident;
		if self.optional {
			quote! {
				#attrs
				#ident: match projection.#ident {
					::core::option::Option::Some(value) => #sea_orm::ActiveValue::Set(value),
					::core::option::Option::None => #sea_orm::ActiveValue::NotSet,
				},
			}
		} else {
			quote!(#attrs #ident: #sea_orm::ActiveValue::Set(projection.#ident),)
		}
	}
}

/// All resolved inputs needed to emit SeaORM support for one projection.
///
/// Every path and type is already resolved by `projected-derive`; this layer
/// deliberately performs no syntax parsing or model-policy decisions.
pub struct ProjectionInput {
	/// Resolved path to the `projected` runtime crate.
	pub runtime: TokenStream,
	/// Resolved path to the `projected-seaorm` integration crate.
	pub integration: TokenStream,
	/// Resolved path to SeaORM.
	pub sea_orm: TokenStream,
	/// Projection type, including generic arguments.
	pub projection_type: TokenStream,
	/// Scalar SeaORM model type completed by the projection.
	pub source_type: TokenStream,
	/// Generated missing-values type, or `()` for a lossless projection.
	pub missing_type: TokenStream,
	/// Whether completing the projection requires a missing-values argument.
	pub has_missing: bool,
	/// Generic parameters for generated implementation blocks.
	pub impl_generics: TokenStream,
	/// Where clause shared with the source model.
	pub where_clause: TokenStream,
	/// Included scalar fields in source declaration order.
	pub fields: Vec<FieldInput>,
}

impl ProjectionInput {
	/// Emits conversion implementations and inherent SeaORM convenience methods.
	///
	/// The generated `From<Projection> for ActiveModel` initializes included
	/// fields explicitly and relies on `ActiveModel::default()` to leave excluded
	/// fields `NotSet`. It also emits `projected_seaorm::SeaOrmProjection`-style
	/// behavior using the resolved integration path, plus `to_model` and
	/// `to_active_model` inherent methods.
	pub fn emit(self) -> TokenStream {
		let ProjectionInput {
			runtime,
			integration,
			sea_orm,
			projection_type,
			source_type,
			missing_type,
			has_missing,
			impl_generics,
			where_clause,
			fields,
		} = self;
		let active_fields = fields.iter().map(|field| field.emit_active_field(&sea_orm));
		let to_model = if has_missing {
			quote! {
				/// Completes this projection into its SeaORM model.
				pub fn to_model(self, missing: #missing_type) -> #source_type {
					<Self as #runtime::Projection>::complete(self, missing)
				}
			}
		} else {
			quote! {
				/// Converts this lossless projection into its SeaORM model.
				pub fn to_model(self) -> #source_type {
					<Self as #runtime::Projection>::complete(self, ())
				}
			}
		};

		quote! {
			impl #impl_generics ::core::convert::From<#projection_type> for ActiveModel #where_clause {
				fn from(projection: #projection_type) -> Self {
					Self {
						#(#active_fields)*
						..::core::default::Default::default()
					}
				}
			}

			impl #impl_generics #integration::SeaOrmProjection for #projection_type #where_clause {
				type ActiveModel = ActiveModel;

				fn into_active_model(self) -> Self::ActiveModel {
					self.into()
				}
			}

			impl #impl_generics #projection_type #where_clause {
				#to_model

				/// Converts this projection into a SeaORM active model.
				pub fn to_active_model(self) -> ActiveModel {
					<#projection_type as #integration::SeaOrmProjection>::into_active_model(self)
				}
			}
		}
	}
}
