use crate::projected::models::raw_field_rule::parse_configurations;
use crate::projected::models::{
	AttributePolicy, FieldAction, FieldTargets, ProjectedOptions, ProjectionDeclaration,
	PropagatedAttribute, RawFieldConfiguration, ResolvedModel, ResolvedProjection, SeaOrmInfo,
	Selection, SourceField,
};
use crate::projected::{names, serde_name};
use proc_macro2::{Ident, Span, TokenStream};
use syn::punctuated::Punctuated;
use syn::{
	Attribute, Data, DeriveInput, Expr, Fields, Generics, Meta, Path, Token, Visibility,
	parenthesized,
};

/// Parsed post-transformation source model and all configuration needed for
/// backend-neutral resolution.
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
	pub fields: Vec<SourceField>,
	/// Struct-level projection declarations in source order.
	pub declarations: Vec<ProjectionDeclaration>,
	/// Module placement and source ordering options.
	pub options: ProjectedOptions,
	/// Derive inheritance and attribute propagation policy.
	pub attributes: AttributePolicy,
	/// SeaORM role inferred from the post-transformation representation.
	pub sea_orm: SeaOrmInfo,
}

impl SourceModel {
	/// Parses the hidden derive input into a source model without emitting tokens.
	///
	/// This is the single normalization point for struct/field configuration,
	/// safe derives, field ordering eligibility, and directional Serde names.
	/// Tuple structs, unit structs, and non-struct inputs are rejected here.
	pub fn parse(input: DeriveInput) -> syn::Result<Self> {
		let fields = match input.data {
			Data::Struct(data) => match data.fields {
				Fields::Named(fields) => fields.named,
				Fields::Unnamed(fields) => {
					return Err(syn::Error::new_spanned(
						fields,
						"projected can only be used on structs with named fields; tuple structs are not supported",
					));
				}
				Fields::Unit => {
					return Err(syn::Error::new_spanned(
						input.ident,
						"projected can only be used on structs with named fields; unit structs are not supported",
					));
				}
			},
			_ => {
				return Err(syn::Error::new_spanned(
					input.ident,
					"projected can only be used on structs with named fields",
				));
			}
		};

		let mut policy = AttributePolicy::default();
		let mut options = ProjectedOptions::default();
		let mut declarations = Vec::new();
		let mut force_emit = false;
		let mut force_sea_orm = false;
		for attr in &input.attrs {
			policy.inherit_safe_derives(attr)?;
			if attr.path().is_ident("projected_internal") && matches!(attr.meta, Meta::List(_)) {
				Self::parse_config(
					attr,
					&mut options,
					&mut policy,
					&mut declarations,
					&mut force_emit,
					&mut force_sea_orm,
				)?;
			}
		}

		let mut source_fields = Vec::with_capacity(fields.len());
		for field in fields {
			let Some(ident) = field.ident else {
				unreachable!("named fields always have identifiers");
			};
			let mut rules = Vec::new();
			let mut orderable = true;
			let mut order_skip_span = None;
			for attr in &field.attrs {
				if !attr.path().is_ident("projected_internal") {
					continue;
				}
				let Meta::List(_) = &attr.meta else {
					return Err(syn::Error::new_spanned(
						attr,
						"expected field-level projected configuration",
					));
				};
				let parsed = attr.parse_args_with(parse_configurations)?;
				if parsed.is_empty() {
					return Err(syn::Error::new_spanned(
						attr,
						"expected at least one field-level projected modifier",
					));
				}
				for configuration in parsed {
					match configuration {
						RawFieldConfiguration::Projection(rule) => rules.push(rule.into_rule()),
						RawFieldConfiguration::OrderSkip(span) => {
							if let Some(previous) = order_skip_span {
								let mut error =
									syn::Error::new(span, "duplicate `order(skip)` configuration");
								error.combine(syn::Error::new(previous, "first configured here"));
								return Err(error);
							}
							order_skip_span = Some(span);
							orderable = false;
						}
					}
				}
			}
			let field_name = ident.to_string().trim_start_matches("r#").to_owned();
			let names = serde_name::resolve(&input.attrs, &field.attrs, &field_name)?;
			source_fields.push(SourceField {
				ident,
				ty: field.ty,
				attrs: field.attrs,
				names,
				orderable,
				rules,
			});
		}

		let sea_orm = Self::detect_sea_orm(&input.ident, &input.attrs, force_emit, force_sea_orm)?;
		Ok(Self {
			ident: input.ident,
			visibility: input.vis,
			generics: input.generics,
			attrs: input.attrs,
			fields: source_fields,
			declarations,
			options,
			attributes: policy,
			sea_orm,
		})
	}

	/// Accumulates one struct-level helper attribute into shared options,
	/// projection declarations, propagation policy, and internal backend flags.
	///
	/// Duplicate singleton options are rejected as soon as they are observed;
	/// repeated projection lists and propagation categories compose.
	fn parse_config(
		attr: &Attribute,
		options: &mut ProjectedOptions,
		policy: &mut AttributePolicy,
		declarations: &mut Vec<ProjectionDeclaration>,
		force_emit: &mut bool,
		force_sea_orm: &mut bool,
	) -> syn::Result<()> {
		attr.parse_nested_meta(|meta| {
			if meta.path.is_ident("projections") {
				let content;
				parenthesized!(content in meta.input);
				let parsed =
					Punctuated::<ProjectionDeclaration, Token![,]>::parse_terminated(&content)?;
				if parsed.is_empty() {
					return Err(meta.error("expected at least one projection declaration"));
				}
				declarations.extend(parsed);
				Ok(())
			} else if meta.path.is_ident("module") {
				if options.module.is_some() {
					return Err(meta.error("duplicate `module` configuration"));
				}
				let module = if meta.input.peek(Token![=]) {
					meta.value()?.parse::<Ident>()?
				} else if meta.input.is_empty() || meta.input.peek(Token![,]) {
					Ident::new(
						"projection",
						meta.path.get_ident().expect("module is an identifier").span(),
					)
				} else {
					return Err(meta.error("expected `module` or `module = identifier`"));
				};
				options.module = Some(module);
				Ok(())
			} else if meta.path.is_ident("projection_derives") {
				let content;
				parenthesized!(content in meta.input);
				let derives = Punctuated::<Path, Token![,]>::parse_terminated(&content)?;
				if derives.is_empty() {
					return Err(meta.error("expected at least one derive path"));
				}
				for derive in derives {
					policy.push_projection_derive(derive);
				}
				Ok(())
			} else if meta.path.is_ident("propagate") {
				let content;
				parenthesized!(content in meta.input);
				let names = Punctuated::<Ident, Token![,]>::parse_terminated(&content)?;
				if names.is_empty() {
					return Err(meta.error("expected at least one propagation category"));
				}
				for name in names {
					let category = if name == "doc" {
						PropagatedAttribute::Doc
					} else if name == "cfg" {
						PropagatedAttribute::Cfg
					} else if name == "cfg_attr" {
						PropagatedAttribute::CfgAttr
					} else if name == "serde" {
						PropagatedAttribute::Serde
					} else if name == "schema" {
						PropagatedAttribute::Schema
					} else {
						return Err(syn::Error::new(
							name.span(),
							"unknown propagation category; expected doc, cfg, cfg_attr, serde, or schema",
						));
					};
					if !policy.allowed.contains(&category) {
						policy.allowed.push(category);
					}
				}
				Ok(())
			} else if meta.path.is_ident("orderable") {
				if options.orderable {
					return Err(meta.error("duplicate `orderable` configuration"));
				}
				options.orderable = true;
				Ok(())
			} else if meta.path.is_ident("emit") {
				if *force_emit {
					return Err(meta.error("duplicate `emit` configuration"));
				}
				*force_emit = true;
				Ok(())
			} else if meta.path.is_ident("sea_orm") {
				if *force_sea_orm {
					return Err(meta.error("duplicate `sea_orm` configuration"));
				}
				*force_sea_orm = true;
				Ok(())
			} else {
				Err(meta.error(
					"unknown projected option; expected projections, module, projection_derives, propagate, orderable, emit, or sea_orm",
				))
			}
		})
	}

	/// Classifies a post-`sea_orm::model` struct as the scalar model, generated
	/// relationship companion, or an unrelated plain struct.
	///
	/// SeaORM marks the scalar model with `sea_orm(model_ex)`. The compatibility
	/// fallback recognizes a metadata-bearing `*Ex` type as the cloned companion
	/// unless hidden `emit` metadata forces generation. `sea_orm` forces scalar
	/// integration when transformation markers are unavailable.
	fn detect_sea_orm(
		ident: &Ident,
		attrs: &[Attribute],
		force_emit: bool,
		force_sea_orm: bool,
	) -> syn::Result<SeaOrmInfo> {
		let mut model_marker = false;
		let mut model_metadata = false;
		for attr in attrs {
			if !attr.path().is_ident("sea_orm") {
				continue;
			}
			attr.parse_nested_meta(|meta| {
				if meta.path.is_ident("model_ex") && meta.input.is_empty() {
					model_marker = true;
				}
				if meta.path.is_ident("table_name") || meta.path.is_ident("schema_name") {
					model_metadata = true;
				}
				Self::consume_meta(meta)
			})?;
		}
		let generated_name = ident.to_string().ends_with("Ex");
		let generated_companion = model_metadata && !model_marker && generated_name && !force_emit;
		Ok(SeaOrmInfo {
			is_model: force_sea_orm || model_marker || (model_metadata && !generated_companion),
			is_generated_companion: generated_companion,
		})
	}

	/// Consumes the unrecognized payload of one SeaORM nested-meta item.
	///
	/// Detection cares only about model markers and table/schema metadata, but
	/// the parser must still advance past values and nested lists correctly.
	fn consume_meta(meta: syn::meta::ParseNestedMeta<'_>) -> syn::Result<()> {
		if meta.input.peek(Token![=]) {
			let _: Expr = meta.value()?.parse()?;
		} else if meta.input.peek(syn::token::Paren) {
			let content;
			parenthesized!(content in meta.input);
			let _: TokenStream = content.parse()?;
		}
		Ok(())
	}

	/// Validates declarations, resolves each projection, then applies field-level
	/// rules to produce an emission-ready model.
	///
	/// Field rules run after declaration modifiers so explicit conflicts retain
	/// both source spans and cannot be hidden by processing order.
	pub fn resolve(self) -> syn::Result<ResolvedModel> {
		self.validate_projection_names()?;
		let mut projections = Vec::with_capacity(self.declarations.len());
		for declaration in &self.declarations {
			projections.push(declaration.resolve(&self)?);
		}
		self.apply_field_rules(&mut projections)?;
		Ok(ResolvedModel {
			source: self,
			projections,
		})
	}

	/// Rejects source/projection name collisions, duplicate declarations, and
	/// declarations that collide with generated missing-values helper names.
	pub fn validate_projection_names(&self) -> syn::Result<()> {
		for (index, declaration) in self.declarations.iter().enumerate() {
			if declaration.ident == self.ident {
				return Err(syn::Error::new(
					declaration.span,
					"a projection name cannot be the same as its source type",
				));
			}
			if let Some(previous) = self.declarations[..index]
				.iter()
				.find(|previous| previous.ident == declaration.ident)
			{
				let mut error = syn::Error::new(
					declaration.span,
					format!("duplicate projection declaration `{}`", declaration.ident),
				);
				error.combine(syn::Error::new(previous.span, "first declared here"));
				return Err(error);
			}
			let helper = names::missing_type(&declaration.ident);
			if let Some(conflicting) = self
				.declarations
				.iter()
				.find(|candidate| candidate.ident == helper)
			{
				let mut error = syn::Error::new(
					conflicting.span,
					format!(
						"projection `{}` conflicts with generated helper type `{helper}`",
						conflicting.ident
					),
				);
				error.combine(syn::Error::new(
					declaration.span,
					"helper name is generated for this projection",
				));
				return Err(error);
			}
		}
		Ok(())
	}

	/// Resolves a non-empty list of source-field identifiers to declaration-order
	/// indices while rejecting duplicates and unknown fields.
	///
	/// Returning stable indices lets every later phase refer to the same shared
	/// `SourceField` record without copying or reparsing its metadata.
	pub fn resolve_field_list(
		&self,
		references: &[Ident],
		modifier_span: Span,
	) -> syn::Result<Vec<usize>> {
		let mut result = Vec::with_capacity(references.len());
		for (reference_index, reference) in references.iter().enumerate() {
			if let Some(previous) = references[..reference_index]
				.iter()
				.find(|previous| *previous == reference)
			{
				let mut error = syn::Error::new(
					reference.span(),
					format!("duplicate field specification `{reference}`"),
				);
				error.combine(syn::Error::new(previous.span(), "first specified here"));
				return Err(error);
			}
			let Some(index) = self
				.fields
				.iter()
				.position(|field| field.ident == *reference)
			else {
				return Err(syn::Error::new(
					reference.span(),
					format!("unknown source field `{reference}` on `{}`", self.ident),
				));
			};
			result.push(index);
		}
		if references.is_empty() {
			return Err(syn::Error::new(
				modifier_span,
				"expected at least one source field",
			));
		}
		Ok(result)
	}

	/// Finds the user-written reference span corresponding to one resolved source
	/// index, falling back to the macro call site only for impossible mismatches.
	pub fn reference_span(&self, references: &[Ident], source_index: usize) -> Span {
		references
			.iter()
			.find(|reference| self.fields[source_index].ident == **reference)
			.map_or_else(Span::call_site, Ident::span)
	}

	/// Applies field-level rules to their declared projection targets.
	///
	/// Bare rules target every projection. Named targets are de-duplicated and
	/// validated before any mutation for that rule, ensuring an error cannot leave
	/// a partially updated resolved model.
	pub fn apply_field_rules(&self, projections: &mut [ResolvedProjection]) -> syn::Result<()> {
		for (source_index, source_field) in self.fields.iter().enumerate() {
			for rule in &source_field.rules {
				let targets = match &rule.targets {
					FieldTargets::All => (0..projections.len()).collect::<Vec<_>>(),
					FieldTargets::Named(names) => {
						let mut targets = Vec::with_capacity(names.len());
						for (name_index, name) in names.iter().enumerate() {
							if let Some(previous) = names[..name_index]
								.iter()
								.find(|previous| *previous == name)
							{
								let mut error = syn::Error::new(
									name.span(),
									format!("duplicate projection reference `{name}`"),
								);
								error.combine(syn::Error::new(
									previous.span(),
									"first referenced here",
								));
								return Err(error);
							}
							let Some(index) = projections
								.iter()
								.position(|projection| projection.ident == *name)
							else {
								return Err(syn::Error::new(
									name.span(),
									format!(
										"unknown projection `{name}`; field-level attributes cannot declare projections"
									),
								));
							};
							targets.push(index);
						}
						targets
					}
				};

				for projection_index in targets {
					let field = &mut projections[projection_index].fields[source_index];
					match rule.action {
						FieldAction::Include => {
							field.apply_selection(Selection::Included, rule.span)?;
						}
						FieldAction::Exclude => {
							field.apply_selection(Selection::Excluded, rule.span)?;
						}
						FieldAction::Optional => {
							field.apply_optionality(rule.span, true)?;
						}
					}
				}
			}
		}
		Ok(())
	}
}
