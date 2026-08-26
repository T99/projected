use proc_macro2::TokenStream;
use syn::{parse_macro_input, ItemStruct};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use crate::projected::options::ProjectedStructOption;

pub struct ProjectedMacroAttribute {
	projected_struct_options: Vec<ProjectedStructOption>,
}

impl ProjectedMacroAttribute {
	
	/// Parses the input and arguments for the `projected` macro attribute,
	/// processes them, and returns the modified token stream.
	///
	/// # Arguments
	/// * `args` - The token stream representing the arguments passed to the
	///   macro attribute.
	/// * `input` - The token stream representing the input to the macro
	///   attribute (e.g., a struct).
	///
	/// # Returns
	/// A token stream representing the modified input after processing the
	/// macro attribute.
	pub fn parse(
		args: proc_macro::TokenStream,
		input: proc_macro::TokenStream,
	) -> proc_macro::TokenStream {
		let args = TokenStream::from(args);
		let mut item = parse_macro_input!(input as ItemStruct);
		match Self::expand(args, &mut item) {
			Ok(()) => quote::quote!(#item).into(),
			Err(error) => error.into_compile_error().into(),
		}
	}
	
	/// Expands the `projected` macro attribute by processing the provided
	/// arguments and modifying the input struct accordingly.
	///
	/// # Arguments
	/// * `args` - The token stream representing the arguments passed to the
	///   macro attribute.
	/// * `input` - A mutable reference to the `ItemStruct` representing the
	///   input struct to be modified.
	///
	/// # Returns
	/// A `syn::Result<()>` indicating success or failure of the expansion.
	fn expand(args: TokenStream, input: &mut ItemStruct) -> syn::Result<()> {
		let projected_macro_attribute = Self {
			projected_struct_options: Self::parse_projected_struct_options(args)?
		};
		Ok(())
	}
	
	fn parse_projected_struct_options(
		args: TokenStream,
	) -> syn::Result<Vec<ProjectedStructOption>> {
		Ok(Punctuated::<ProjectedStructOption, syn::Token![,]>::parse_terminated.parse2(args)?
			.into_iter()
			.collect())
	}
}