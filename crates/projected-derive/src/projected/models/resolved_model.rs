use crate::projected::codegen::{MetadataField, MetadataInput, OrderField, OrderInput};
use crate::projected::models::{ResolvedProjection, SourceModel};
use crate::projected::{attributes, names};
use crate::util::real_crate_path;
use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::ext::IdentExt;

/// Dependency paths required only when emitting SeaORM integration tokens.
pub struct SeaOrmPaths {
	/// Resolved path to the consumer's direct `sea-orm` dependency.
	pub sea_orm: TokenStream,
	/// Resolved path to the consumer's direct `projected-seaorm` dependency.
	pub integration: TokenStream,
}

/// Fully validated model ready for deterministic token generation.
pub struct ResolvedModel {
	/// Parsed source model and shared field metadata.
	pub source: SourceModel,
	/// Resolved projections in declaration order.
	pub projections: Vec<ResolvedProjection>,
}

impl ResolvedModel {
	/// Emits source metadata/order support and every declared projection.
	///
	/// SeaORM dependencies are resolved only for a detected scalar SeaORM model,
	/// keeping plain users ORM-agnostic. Projection output is optionally wrapped
	/// in the configured visibility-preserving module.
	pub fn emit(&self) -> syn::Result<TokenStream> {
		let runtime = real_crate_path("projected")?;
		let sea_orm = self
			.source
			.sea_orm
			.is_model
			.then(|| {
				Ok::<SeaOrmPaths, syn::Error>(SeaOrmPaths {
					sea_orm: real_crate_path("sea-orm")?,
					integration: real_crate_path("projected-seaorm")?,
				})
			})
			.transpose()?;
		let source_items = self.emit_source_items(&runtime);
		let projections = self
			.projections
			.iter()
			.map(|projection| projection.emit(self, &runtime, sea_orm.as_ref()))
			.collect::<syn::Result<Vec<_>>>()?;
		let projection_items = quote!(#(#projections)*);
		let placed_projections = if let Some(module) = &self.source.options.module {
			let visibility = &self.source.visibility;
			let source = &self.source.ident;
			let module_doc = format!("Projections generated for [`{source}`].");
			quote! {
				#[doc = #module_doc]
				#visibility mod #module {
					use super::*;
					#projection_items
				}
			}
		} else {
			projection_items
		};
		Ok(quote! {
			#source_items
			#placed_projections
		})
	}

	/// Emits the source model's shared field metadata and optional ordering API.
	///
	/// Both emitters consume the same `SourceField` names and eligibility flags;
	/// neither reparses source attributes independently.
	fn emit_source_items(&self, runtime: &TokenStream) -> TokenStream {
		let source = &self.source;
		let source_ident = &source.ident;
		let field_ident = names::field_type(source_ident);
		let visibility = &source.visibility;
		let generics = &source.generics;
		let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
		let source_type = quote!(#source_ident #ty_generics);
		let metadata_fields = source
			.fields
			.iter()
			.map(|field| {
				let conditional = attributes::conditional(&field.attrs);
				let rust_name = field.ident.unraw().to_string();
				let variant = format_ident!("{}", rust_name.to_upper_camel_case());
				MetadataField {
					attrs: quote!(#(#conditional)*),
					variant: variant.clone(),
					source_variant: variant,
					rust_name,
					serialize_name: field.names.serialize.clone(),
					deserialize_name: field.names.deserialize.clone(),
					orderable: field.orderable,
				}
			})
			.collect();
		let metadata = MetadataInput {
			runtime: runtime.clone(),
			visibility: quote!(#visibility),
			model_type: source_type.clone(),
			field_ident: field_ident.clone(),
			source_type: source_type.clone(),
			source_field_ident: field_ident.clone(),
			impl_generics: quote!(#impl_generics),
			where_clause: quote!(#where_clause),
			fields: metadata_fields,
		}
		.emit();
		let orderable = source.options.orderable.then(|| {
			let fields = source
				.fields
				.iter()
				.filter(|field| field.orderable)
				.map(|field| {
					let conditional = attributes::conditional(&field.attrs);
					OrderField {
						attrs: quote!(#(#conditional)*),
						rust_name: field.ident.unraw().to_string(),
						serialize_name: field.names.serialize.clone(),
						deserialize_name: field.names.deserialize.clone(),
					}
				})
				.collect();
			OrderInput {
				runtime: runtime.clone(),
				visibility: quote!(#visibility),
				model_type: source_type,
				model_ident: source_ident.clone(),
				field_ident,
				impl_generics: quote!(#impl_generics),
				where_clause: quote!(#where_clause),
				fields,
			}
			.emit()
		});
		quote! {
			#metadata
			#orderable
		}
	}
}
