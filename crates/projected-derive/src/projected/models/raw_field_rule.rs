use crate::projected::models::{FieldAction, FieldRule, FieldTargets};
use proc_macro2::{Ident, Span};
use syn::parse::{Parse, ParseStream};
use syn::{Token, parenthesized};

/// Raw field-level configuration parsed from hidden `projected_internal`
/// metadata before projection targets are validated.
pub enum RawFieldConfiguration {
	/// A projection selection or optionality rule.
	Projection(RawFieldRule),
	/// An ordering opt-out with its diagnostic span.
	OrderSkip(Span),
}

/// Unresolved field-level projection rule.
pub struct RawFieldRule {
	/// Parsed include/exclude/optional action.
	action: FieldAction,
	/// Action keyword span used for diagnostics.
	span: Span,
	/// Optional projection targets; absence means every declared projection.
	targets: Option<Vec<Ident>>,
}

impl RawFieldRule {
	/// Converts raw optional targets into the normalized all-or-named target
	/// representation without validating declaration names yet.
	pub fn into_rule(self) -> FieldRule {
		FieldRule {
			action: self.action,
			targets: self.targets.map_or(FieldTargets::All, FieldTargets::Named),
			span: self.span,
		}
	}
}

impl Parse for RawFieldConfiguration {
	/// Parses either `order(skip)` or an include/exclude/optional field rule.
	fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
		let modifier: Ident = input.parse()?;
		if modifier == "order" {
			let content;
			parenthesized!(content in input);
			let skip: Ident = content.parse()?;
			if skip != "skip" || !content.is_empty() {
				return Err(syn::Error::new(skip.span(), "expected `order(skip)`"));
			}
			return Ok(Self::OrderSkip(modifier.span()));
		}
		let action = FieldAction::parse_from_ident(&modifier)?;
		let targets = input
			.peek(syn::token::Paren)
			.then(|| crate::util::parse_ident_list(input, "projection name"))
			.transpose()?;
		Ok(Self::Projection(RawFieldRule {
			action,
			span: modifier.span(),
			targets,
		}))
	}
}

/// Parses all comma-separated configurations in one field helper attribute.
pub fn parse_configurations(
	input: syn::parse::ParseStream<'_>,
) -> syn::Result<syn::punctuated::Punctuated<RawFieldConfiguration, Token![,]>> {
	syn::punctuated::Punctuated::<RawFieldConfiguration, Token![,]>::parse_terminated(input)
}
