mod projected;
pub mod util;

use proc_macro::TokenStream;
use projected::projected_macro_attribute::ProjectedMacroAttribute;

#[proc_macro_attribute]
pub fn projected(args: TokenStream, input: TokenStream) -> TokenStream {
	ProjectedMacroAttribute::parse(args, input)
}