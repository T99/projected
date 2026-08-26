use proc_macro2::Ident;

/// Options global to a given source model marked as `#[projected(...)]`.
/// 
/// 
pub struct SourceModelOptions {
	pub module: Option<Ident>,
}