use proc_macro2::Ident;

/// Selection or optionality action shared by struct- and field-level rules.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FieldAction {
	/// Select a source field for a projection.
	Include,
	/// Remove a source field from a projection.
	Exclude,
	/// Make a selected field omission-aware by adding an outer `Option`.
	Optional,
}

impl FieldAction {
	/// Parses a projection action keyword and reports the accepted vocabulary on
	/// unknown identifiers.
	pub fn parse_from_ident(ident: &Ident) -> syn::Result<FieldAction> {
		if ident == "include" {
			Ok(FieldAction::Include)
		} else if ident == "exclude" {
			Ok(FieldAction::Exclude)
		} else if ident == "optional" {
			Ok(FieldAction::Optional)
		} else {
			Err(syn::Error::new(
				ident.span(),
				"unknown projection modifier; expected `include`, `exclude`, or `optional`",
			))
		}
	}
}
