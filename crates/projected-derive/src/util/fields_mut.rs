use syn::{Field, Fields};

/// Provides one uniform mutable iterator over every struct field shape.
///
/// Unsupported tuple and unit structs are diagnosed in the later derive phase;
/// this phase still needs to preserve them so the error points at the item.
pub fn fields_mut(fields: &mut Fields) -> Box<dyn Iterator<Item = &mut Field> + '_> {
	match fields {
		Fields::Named(fields) => Box::new(fields.named.iter_mut()),
		Fields::Unnamed(fields) => Box::new(fields.unnamed.iter_mut()),
		Fields::Unit => Box::new(std::iter::empty()),
	}
}