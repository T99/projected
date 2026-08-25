use crate::projected::models::{FieldAction, ProjectionModifier};
use proc_macro2::{Ident, Span};
use syn::parse::{Parse, ParseStream};

/// Struct-level projection modifier before it is normalized into separate
/// selection and optionality forms.
pub struct RawProjectionModifier {
	/// Parsed include/exclude/optional action.
	action: FieldAction,
	/// Action keyword span used for diagnostics.
	span: Span,
	/// Optional source field list; only bare `optional` may omit it.
	fields: Option<Vec<Ident>>,
}

impl RawProjectionModifier {
	/// Converts parser output to the normalized modifier used during resolution.
	///
	/// Parser validation guarantees that include/exclude always carry field
	/// lists, making the remaining fallback unreachable.
	pub fn into_modifier(self) -> ProjectionModifier {
		match (self.action, self.fields) {
			(FieldAction::Include, Some(fields)) => ProjectionModifier::Include {
				span: self.span,
				fields,
			},
			(FieldAction::Exclude, Some(fields)) => ProjectionModifier::Exclude {
				span: self.span,
				fields,
			},
			(FieldAction::Optional, Some(fields)) => ProjectionModifier::OptionalFields {
				span: self.span,
				fields,
			},
			(FieldAction::Optional, None) => ProjectionModifier::OptionalAll { span: self.span },
			_ => unreachable!("include and exclude require field lists"),
		}
	}
}

impl Parse for RawProjectionModifier {
	/// Parses `include(fields)`, `exclude(fields)`, `optional(fields)`, or bare
	/// `optional` with precise missing-list diagnostics.
	fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
		let modifier: Ident = input.parse()?;
		let action = FieldAction::parse_from_ident(&modifier)?;
		let fields = if input.peek(syn::token::Paren) {
			Some(crate::util::parse_ident_list(input, "source field")?)
		} else if action == FieldAction::Optional {
			None
		} else {
			return Err(syn::Error::new(
				modifier.span(),
				format!("`{modifier}` requires a parenthesized field list"),
			));
		};
		Ok(Self {
			action,
			span: modifier.span(),
			fields,
		})
	}
}
