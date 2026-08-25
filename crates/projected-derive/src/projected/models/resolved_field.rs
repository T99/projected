use crate::projected::models::{RuleOrigin, Selection};
use proc_macro2::Span;

/// Per-projection state for one source field after declaration defaults and
/// explicit rules are applied.
pub struct ResolvedField {
	/// Index into `SourceModel::fields`, preserving declaration order.
	pub source_index: usize,
	/// Whether the projection contains the field.
	pub selection: Selection,
	/// Origin of the current selection for conflict reporting.
	pub selection_origin: RuleOrigin,
	/// Whether the projection adds an outer omission-aware `Option`.
	pub optional: bool,
	/// Span of the explicit optionality rule, when present.
	pub optional_origin: Option<Span>,
}

impl ResolvedField {
	/// Applies an explicit selection and rejects duplicate or conflicting
	/// explicit selection rules.
	///
	/// Implicit declaration defaults may be replaced once without conflict.
	pub fn apply_selection(&mut self, selection: Selection, span: Span) -> syn::Result<()> {
		if let Some(previous) = self.selection_origin.explicit_span() {
			let message = if self.selection == selection {
				"duplicate explicit selection rule for this field and projection"
			} else {
				"conflicting explicit selection rules for this field and projection"
			};
			let mut error = syn::Error::new(span, message);
			error.combine(syn::Error::new(previous, "previous explicit rule is here"));
			return Err(error);
		}
		self.selection = selection;
		self.selection_origin = RuleOrigin::Explicit(span);
		Ok(())
	}

	/// Makes this field projection-optional, optionally including an implicitly
	/// excluded field first.
	///
	/// Explicit exclusions cannot be overridden by optionality, and a second
	/// explicit optional rule is always diagnosed with both source spans.
	pub fn apply_optionality(&mut self, span: Span, include_if_necessary: bool) -> syn::Result<()> {
		if let Some(previous) = self.optional_origin {
			let mut error = syn::Error::new(
				span,
				"duplicate explicit optional rule for this field and projection",
			);
			error.combine(syn::Error::new(previous, "previous optional rule is here"));
			return Err(error);
		}
		if self.selection == Selection::Excluded {
			if !include_if_necessary {
				return Ok(());
			}
			if let Some(previous) = self.selection_origin.explicit_span() {
				let mut error = syn::Error::new(
					span,
					"`optional` cannot include a field that an explicit rule excludes",
				);
				error.combine(syn::Error::new(previous, "explicit exclusion is here"));
				return Err(error);
			}
			self.selection = Selection::Included;
			self.selection_origin = RuleOrigin::Explicit(span);
		}
		self.optional = true;
		self.optional_origin = Some(span);
		Ok(())
	}
}
