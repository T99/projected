/// Resolves Cargo dependency names for generated paths.
mod crate_path;
pub use crate_path::real_crate_path;

mod fields_mut;
pub use fields_mut::fields_mut;

/// Detects which source generics are carried by generated fields.
mod generic_usage;
pub use generic_usage::uses_all_generic_parameters;

/// Parses non-empty identifier lists shared by configuration forms.
mod parse_ident_list;
pub use parse_ident_list::parse_ident_list;
