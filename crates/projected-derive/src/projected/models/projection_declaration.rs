use crate::projected::models::{
	ProjectionModifier, RawProjectionModifier, ResolvedField, ResolvedProjection, RuleOrigin,
	Selection, SourceModel,
};
use proc_macro2::{Ident, Span};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Token, parenthesized};

/// One named projection and its normalized struct-level modifiers.
pub struct ProjectionDeclaration {
	/// Generated projection type identifier.
	pub ident: Ident,
	/// Declaration span used for duplicate/name-conflict diagnostics.
	pub span: Span,
	/// Selection and optionality modifiers in source order.
	pub modifiers: Vec<ProjectionModifier>,
}

impl ProjectionDeclaration {
	/// Resolves declaration defaults and modifiers into one state entry per
	/// source field.
	///
	/// Selection is resolved before optionality so named optional fields may
	/// include an implicitly excluded field but cannot override an explicit
	/// exclusion. Mixing include-mode and exclude-mode is rejected up front.
	pub fn resolve(&self, source: &SourceModel) -> syn::Result<ResolvedProjection> {
		let include_span = self.modifiers.iter().find_map(|modifier| match modifier {
			ProjectionModifier::Include { span, .. } => Some(*span),
			_ => None,
		});
		let exclude_span = self.modifiers.iter().find_map(|modifier| match modifier {
			ProjectionModifier::Exclude { span, .. } => Some(*span),
			_ => None,
		});
		if let (Some(include), Some(exclude)) = (include_span, exclude_span) {
			let mut error = syn::Error::new(
				exclude,
				"a projection cannot use both `include` and `exclude` selection rules",
			);
			error.combine(syn::Error::new(include, "the `include` rule is here"));
			return Err(error);
		}

		let initial = if include_span.is_some() {
			Selection::Excluded
		} else {
			Selection::Included
		};
		let mut fields = (0..source.fields.len())
			.map(|source_index| ResolvedField {
				source_index,
				selection: initial,
				selection_origin: RuleOrigin::Implicit,
				optional: false,
				optional_origin: None,
			})
			.collect::<Vec<_>>();

		for modifier in &self.modifiers {
			match modifier {
				ProjectionModifier::Include {
					span,
					fields: references,
				} => {
					for index in source.resolve_field_list(references, *span)? {
						fields[index].apply_selection(
							Selection::Included,
							source.reference_span(references, index),
						)?;
					}
				}
				ProjectionModifier::Exclude {
					span,
					fields: references,
				} => {
					for index in source.resolve_field_list(references, *span)? {
						fields[index].apply_selection(
							Selection::Excluded,
							source.reference_span(references, index),
						)?;
					}
				}
				ProjectionModifier::OptionalAll { .. }
				| ProjectionModifier::OptionalFields { .. } => {}
			}
		}

		for modifier in &self.modifiers {
			match modifier {
				ProjectionModifier::OptionalAll { span } => {
					for field in fields
						.iter_mut()
						.filter(|field| field.selection == Selection::Included)
					{
						field.apply_optionality(*span, false)?;
					}
				}
				ProjectionModifier::OptionalFields {
					span,
					fields: references,
				} => {
					for index in source.resolve_field_list(references, *span)? {
						let field_span = source.reference_span(references, index);
						fields[index].apply_optionality(field_span, true)?;
					}
				}
				ProjectionModifier::Include { .. } | ProjectionModifier::Exclude { .. } => {}
			}
		}

		Ok(ResolvedProjection {
			ident: self.ident.clone(),
			fields,
		})
	}
}

impl Parse for ProjectionDeclaration {
	/// Parses a projection name followed by an optional non-empty modifier list.
	fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
		let ident: Ident = input.parse()?;
		let span = ident.span();
		let modifiers = if input.peek(syn::token::Paren) {
			let content;
			parenthesized!(content in input);
			let parsed =
				Punctuated::<RawProjectionModifier, Token![,]>::parse_terminated(&content)?;
			if parsed.is_empty() {
				return Err(syn::Error::new(
					content.span(),
					"expected a projection modifier",
				));
			}
			parsed
				.into_iter()
				.map(RawProjectionModifier::into_modifier)
				.collect()
		} else {
			Vec::new()
		};
		Ok(Self {
			ident,
			span,
			modifiers,
		})
	}
}
