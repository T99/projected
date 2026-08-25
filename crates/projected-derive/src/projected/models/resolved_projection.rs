use crate::projected::codegen::{MetadataField, MetadataInput, OrderField, OrderInput};
use crate::projected::models::{ResolvedField, ResolvedModel, SeaOrmPaths, Selection, SourceField};
use crate::projected::{attributes, names};
use crate::util::uses_all_generic_parameters;
use heck::ToUpperCamelCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::Generics;
use syn::ext::IdentExt;

/// One projection with final selection and optionality state for every source
/// field.
pub struct ResolvedProjection {
	/// Generated projection type identifier.
	pub ident: Ident,
	/// Per-source-field state in source declaration order.
	pub fields: Vec<ResolvedField>,
}

impl ResolvedProjection {
	/// Emits a projection's value type, missing-values type, completion and
	/// conversion APIs, shared field metadata, optional ordering, and optional
	/// SeaORM integration.
	///
	/// All output derives from the already resolved model. Included and missing
	/// fields retain source order and conditional attributes; generic marker
	/// fields are introduced only when generated data would otherwise leave a
	/// source generic parameter unused.
	pub fn emit(
		&self,
		model: &ResolvedModel,
		runtime: &TokenStream,
		sea_orm: Option<&SeaOrmPaths>,
	) -> syn::Result<TokenStream> {
		let source = &model.source;
		let source_ident = &source.ident;
		let source_field_ident = names::field_type(source_ident);
		let projection_ident = &self.ident;
		let projection_field_ident = names::field_type(projection_ident);
		let missing_ident = names::missing_type(projection_ident);
		let visibility = &source.visibility;
		let generics = &source.generics;
		let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
		let source_type = quote!(#source_ident #ty_generics);
		let projection_type = quote!(#projection_ident #ty_generics);
		let struct_attrs = source.attributes.propagated(&source.attrs);
		let derives = &source.attributes.projection_derives;
		let derive_attr = (!derives.is_empty()).then(|| quote!(#[derive(#(#derives),*)]));
		let projection_doc = format!("Projection of [`{source_ident}`].");

		let included = self
			.fields
			.iter()
			.filter(|field| field.selection == Selection::Included)
			.collect::<Vec<_>>();
		let missing = self
			.fields
			.iter()
			.filter(|field| field.selection == Selection::Excluded || field.optional)
			.collect::<Vec<_>>();

		let marker_ident = names::marker_field(&source.fields);
		let projection_needs_marker = Self::needs_generic_marker(
			generics,
			included
				.iter()
				.map(|resolved| &source.fields[resolved.source_index]),
		);
		let missing_needs_marker = !missing.is_empty()
			&& Self::needs_generic_marker(
				generics,
				missing
					.iter()
					.map(|resolved| &source.fields[resolved.source_index]),
			);

		let projection_fields = included.iter().map(|resolved| {
			let field = &source.fields[resolved.source_index];
			let attrs = source.attributes.propagated(&field.attrs);
			let ident = &field.ident;
			let ty = &field.ty;
			let docs = format!("Projected value of the source `{ident}` field.");
			if resolved.optional {
				quote!(#(#attrs)* #[doc = #docs] pub #ident: ::core::option::Option<#ty>,)
			} else {
				quote!(#(#attrs)* #[doc = #docs] pub #ident: #ty,)
			}
		});
		let projection_marker = projection_needs_marker.then(|| {
			quote!(
				#[doc(hidden)]
				pub #marker_ident: ::core::marker::PhantomData<fn() -> #source_type>,
			)
		});
		let projection_struct = quote! {
			#derive_attr
			#(#struct_attrs)*
			#[doc = #projection_doc]
			#visibility struct #projection_ident #generics #where_clause {
				#(#projection_fields)*
				#projection_marker
			}
		};

		let (missing_struct, missing_type) = if missing.is_empty() {
			(TokenStream::new(), quote!(()))
		} else {
			let missing_doc = format!(
				"Values required to complete [`{projection_ident}`] into [`{source_ident}`]."
			);
			let missing_fields = missing.iter().map(|resolved| {
				let field = &source.fields[resolved.source_index];
				let attrs = source.attributes.propagated(&field.attrs);
				let ident = &field.ident;
				let ty = &field.ty;
				let docs = format!(
					"Value used for the source `{ident}` field when completing the projection."
				);
				quote!(#(#attrs)* #[doc = #docs] pub #ident: #ty,)
			});
			let missing_marker = missing_needs_marker.then(|| {
				quote!(
					#[doc(hidden)]
					pub #marker_ident: ::core::marker::PhantomData<fn() -> #source_type>,
				)
			});
			(
				quote! {
					#derive_attr
					#(#struct_attrs)*
					#[doc = #missing_doc]
					#visibility struct #missing_ident #generics #where_clause {
						#(#missing_fields)*
						#missing_marker
					}
				},
				quote!(#missing_ident #ty_generics),
			)
		};

		let base_initializers = self.fields.iter().map(|resolved| {
			let source_field = &source.fields[resolved.source_index];
			let cfg = attributes::conditional(&source_field.attrs);
			let ident = &source_field.ident;
			match (resolved.selection, resolved.optional) {
				(Selection::Included, false) => quote!(#(#cfg)* #ident: self.#ident,),
				(Selection::Included, true) => quote!(
					#(#cfg)*
					#ident: match self.#ident {
						::core::option::Option::Some(value) => value,
						::core::option::Option::None => missing.#ident,
					},
				),
				(Selection::Excluded, _) => quote!(#(#cfg)* #ident: missing.#ident,),
			}
		});
		let projection_impl = quote! {
			impl #impl_generics #runtime::Projection for #projection_type #where_clause {
				type Base = #source_type;
				type Missing = #missing_type;

				fn complete(self, missing: Self::Missing) -> Self::Base {
					#source_ident {
						#(#base_initializers)*
					}
				}
			}
		};

		let base_projection_fields = included.iter().map(|resolved| {
			let field = &source.fields[resolved.source_index];
			let cfg = attributes::conditional(&field.attrs);
			let ident = &field.ident;
			if resolved.optional {
				quote!(#(#cfg)* #ident: ::core::option::Option::Some(base.#ident),)
			} else {
				quote!(#(#cfg)* #ident: base.#ident,)
			}
		});
		let base_projection_marker =
			projection_needs_marker.then(|| quote!(#marker_ident: ::core::marker::PhantomData,));
		let from_base = quote! {
			impl #impl_generics ::core::convert::From<#source_type> for #projection_type #where_clause {
				fn from(base: #source_type) -> Self {
					Self {
						#(#base_projection_fields)*
						#base_projection_marker
					}
				}
			}
		};

		let (completion_methods, missing_initializer) = self.emit_completion_methods(
			model,
			&missing,
			&missing_ident,
			missing_needs_marker,
			&marker_ident,
			runtime,
			&source_type,
			&missing_type,
		);
		let into_base = missing.is_empty().then(|| {
			quote! {
				impl #impl_generics ::core::convert::From<#projection_type> for #source_type #where_clause {
					fn from(projection: #projection_type) -> Self {
						<#projection_type as #runtime::Projection>::complete(projection, ())
					}
				}
			}
		});

		let metadata_fields = included
			.iter()
			.map(|resolved| {
				let field = &source.fields[resolved.source_index];
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
			model_type: projection_type.clone(),
			field_ident: projection_field_ident.clone(),
			source_type: source_type.clone(),
			source_field_ident,
			impl_generics: quote!(#impl_generics),
			where_clause: quote!(#where_clause),
			fields: metadata_fields,
		}
		.emit();
		let orderable = source.attributes.projections_orderable.then(|| {
			let fields = included
				.iter()
				.filter_map(|resolved| {
					let field = &source.fields[resolved.source_index];
					field.orderable.then(|| {
						let conditional = attributes::conditional(&field.attrs);
						OrderField {
							attrs: quote!(#(#conditional)*),
							rust_name: field.ident.unraw().to_string(),
							serialize_name: field.names.serialize.clone(),
							deserialize_name: field.names.deserialize.clone(),
						}
					})
				})
				.collect();
			OrderInput {
				runtime: runtime.clone(),
				visibility: quote!(#visibility),
				model_type: projection_type.clone(),
				model_ident: projection_ident.clone(),
				field_ident: projection_field_ident,
				impl_generics: quote!(#impl_generics),
				where_clause: quote!(#where_clause),
				fields,
			}
			.emit()
		});
		let sea_orm_tokens = sea_orm.map(|paths| {
			self.emit_sea_orm(
				model,
				paths,
				runtime,
				&projection_type,
				&source_type,
				&missing_type,
				!missing.is_empty(),
				quote!(#impl_generics),
				quote!(#where_clause),
			)
		});

		Ok(quote! {
			#projection_struct
			#missing_struct
			#metadata
			#orderable
			#projection_impl
			#from_base
			impl #impl_generics #projection_type #where_clause {
				#completion_methods
				#missing_initializer
			}
			#into_base
			#sea_orm_tokens
		})
	}

	/// Returns whether generated fields fail to carry one or more source generic
	/// parameters in every compilation configuration.
	///
	/// A conditional field is excluded from the proof because it may disappear;
	/// when any generic is unrepresented, a hidden invariant `PhantomData` field
	/// preserves well-formed generic usage without imposing ownership semantics.
	fn needs_generic_marker<'a>(
		generics: &Generics,
		fields: impl Iterator<Item = &'a SourceField>,
	) -> bool {
		if generics.params.is_empty() {
			return false;
		}
		!uses_all_generic_parameters(
			generics,
			fields
				.filter(|field| !attributes::may_remove_item(&field.attrs))
				.map(|field| &field.ty),
		)
	}

	/// Emits ergonomic completion methods for either a lossless or lossy
	/// projection.
	///
	/// Lossless projections receive zero-argument `complete_with` and `into_base`.
	/// Lossy projections receive source-ordered completion parameters plus a
	/// method accepting the generated missing-values struct. Optional projection
	/// fields use collision-safe fallback parameter names.
	#[expect(clippy::too_many_arguments)]
	fn emit_completion_methods(
		&self,
		model: &ResolvedModel,
		missing: &[&ResolvedField],
		missing_ident: &Ident,
		missing_needs_marker: bool,
		marker_ident: &Ident,
		runtime: &TokenStream,
		source_type: &TokenStream,
		missing_type: &TokenStream,
	) -> (TokenStream, TokenStream) {
		let projection_ident = &self.ident;
		if missing.is_empty() {
			return (
				quote! {
					/// Completes this projection without additional values.
					pub fn complete_with(self) -> #source_type {
						<Self as #runtime::Projection>::complete(self, ())
					}

					/// Converts this lossless projection into its base value.
					pub fn into_base(self) -> #source_type {
						<Self as #runtime::Projection>::complete(self, ())
					}
				},
				TokenStream::new(),
			);
		}

		let source = &model.source;
		let mut occupied = missing
			.iter()
			.filter(|resolved| resolved.selection == Selection::Excluded)
			.map(|resolved| source.fields[resolved.source_index].ident.to_string())
			.collect::<Vec<_>>();
		let mut parameters = Vec::with_capacity(missing.len());
		let mut initializers = Vec::with_capacity(missing.len());
		for resolved in missing {
			let field = &source.fields[resolved.source_index];
			let cfg = attributes::conditional(&field.attrs);
			let field_ident = &field.ident;
			let ty = &field.ty;
			let parameter = if resolved.selection == Selection::Included && resolved.optional {
				names::fallback_parameter(field_ident, &mut occupied)
			} else {
				field_ident.clone()
			};
			parameters.push(quote!(#(#cfg)* #parameter: #ty));
			initializers.push(quote!(#(#cfg)* #field_ident: #parameter,));
		}
		let marker =
			missing_needs_marker.then(|| quote!(#marker_ident: ::core::marker::PhantomData,));
		let docs = format!(
			"Completes [`{projection_ident}`] from a source-ordered list of omitted values and fallbacks."
		);
		(
			quote! {
				#[doc = #docs]
				pub fn complete_with(self, #(#parameters),*) -> #source_type {
					<Self as #runtime::Projection>::complete(
						self,
						#missing_ident {
							#(#initializers)*
							#marker
						},
					)
				}
			},
			quote! {
				/// Completes this projection from its generated missing-values representation.
				pub fn complete(self, missing: #missing_type) -> #source_type {
					<Self as #runtime::Projection>::complete(self, missing)
				}
			},
		)
	}

	/// Adapts resolved scalar projection fields into the narrow input consumed by
	/// `projected-seaorm-codegen`.
	///
	/// Excluded fields are intentionally absent so the generated `ActiveModel`
	/// initializer leaves them `NotSet` via its default tail expression.
	#[expect(clippy::too_many_arguments)]
	fn emit_sea_orm(
		&self,
		model: &ResolvedModel,
		paths: &SeaOrmPaths,
		runtime: &TokenStream,
		projection_type: &TokenStream,
		source_type: &TokenStream,
		missing_type: &TokenStream,
		has_missing: bool,
		impl_generics: TokenStream,
		where_clause: TokenStream,
	) -> TokenStream {
		let source = &model.source;
		let fields = self
			.fields
			.iter()
			.filter(|resolved| resolved.selection == Selection::Included)
			.map(|resolved| {
				let field = &source.fields[resolved.source_index];
				let conditional = attributes::conditional(&field.attrs);
				projected_seaorm_codegen::FieldInput {
					attrs: quote!(#(#conditional)*),
					ident: field.ident.clone(),
					optional: resolved.optional,
				}
			})
			.collect();
		projected_seaorm_codegen::ProjectionInput {
			runtime: runtime.clone(),
			integration: paths.integration.clone(),
			sea_orm: paths.sea_orm.clone(),
			projection_type: projection_type.clone(),
			source_type: source_type.clone(),
			missing_type: missing_type.clone(),
			has_missing,
			impl_generics,
			where_clause,
			fields,
		}
		.emit()
	}
}
