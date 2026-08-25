use crate::projected::models::SourceField;
use quote::format_ident;
use syn::Ident;

/// Returns the generated backend-neutral field enum name for a model or
/// projection type.
pub fn field_type(model: &Ident) -> Ident {
	format_ident!("{}Field", model)
}

/// Returns the generated Serde-facing ordering enum name for a model.
pub fn order_field_type(model: &Ident) -> Ident {
	format_ident!("{}OrderField", model)
}

/// Returns the generated missing-values struct name for a projection.
pub fn missing_type(projection: &Ident) -> Ident {
	format_ident!("{}Missing", projection)
}

/// Chooses a hidden generic-marker field name that cannot collide with a source
/// field, extending the conventional name with underscores as needed.
pub fn marker_field(source_fields: &[SourceField]) -> Ident {
	let mut candidate = "__projected_marker".to_owned();
	while source_fields.iter().any(|field| field.ident == candidate) {
		candidate.push('_');
	}
	format_ident!("{candidate}")
}

/// Chooses a readable completion fallback parameter without colliding with
/// omitted field parameters already present in the same method signature.
///
/// The result retains the source field's span so any downstream type error is
/// reported near the corresponding field declaration.
pub fn fallback_parameter(field: &Ident, occupied: &mut Vec<String>) -> Ident {
	let mut candidate = format!("{field}_fallback");
	while occupied.iter().any(|name| name == &candidate) {
		candidate.push('_');
	}
	occupied.push(candidate.clone());
	format_ident!("{candidate}", span = field.span())
}
