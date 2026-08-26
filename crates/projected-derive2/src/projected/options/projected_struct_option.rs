use proc_macro2::{Ident, Span};
use syn::{parenthesized, Path, Token, token};
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use crate::projected::ProjectionDeclaration;

/// An enumeration of the possible options available when using the
/// `#[projected(...)]` attribute on a struct.
///
/// # Examples
/// ```rust,ignore
/// #[projected(
///   module = my_module,
///   projection_derives(Serialize, Deserialize),
///   projections(
///     projection1(include(field1, field2), exclude(field3)),
///     projection2(include(field4), optional(field5)),
///   ),
/// )]
/// struct MyStruct { /* ... */}
/// ```
pub enum ProjectedStructOption {
	/// Specifies the module in which the projected structs will be generated.
	///
	/// If no module is specified, the projected structs will be generated in
	/// the same module as the original struct.
	Module {
		/// The span over this option, used for diagnostics.
		span: Span,
		/// The identifier of the module in which the projected structs will be
		/// generated.
		///
		/// `None` if no module is specified, in which case the projected
		/// structs will be generated in the same module as the original struct.
		ident: Option<Ident>,
	},
	/// Specifies the derives to apply to the generated projected structs.
	ProjectionDerives {
		/// The span over this option, used for diagnostics.
		span: Span,
		/// The list of derives to apply to the generated projected structs.
		derives: Vec<Path>,
	},
	/// Specifies the projections to generate for the struct, along with their
	/// associated options.
	Projections {
		/// The span over this option, used for diagnostics.
		span: Span,
		/// The list of projection declarations, each specifying a projection
		/// and its associated options.
		projections: Vec<ProjectionDeclaration>,
	},
}

impl Parse for ProjectedStructOption {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let span = input.span();
		let name: Ident = input.parse()?;
		if name == "module" {
			let ident = if input.peek(Token![=]) {
				input.parse::<Token![=]>()?;
				Some(input.parse()?)
			} else if input.is_empty() || input.peek(Token![,]) {
				None
			} else {
				return Err(syn::Error::new(
					span,
					"expected `module` or `module = identifier`",
				));
			};
			Ok(Self::Module { span, ident })
		} else if name == "projection_derives" {
			let content;
			parenthesized!(content in input);
			let derives = Punctuated::<Path, Token![,]>::parse_terminated(&content)?
				.into_iter()
				.collect::<Vec<_>>();
			if derives.is_empty() {
				return Err(syn::Error::new(
					span,
					"expected at least one derive",
				));
			}
			Ok(Self::ProjectionDerives { span, derives })
		} else if name == "projections" {
			let content;
			parenthesized!(content in input);
			let projections = Punctuated::<ProjectionDeclaration, token::Comma>::parse_terminated(&content)?
				.into_iter()
				.collect::<Vec<_>>();
			if projections.is_empty() {
				return Err(syn::Error::new(
					span,
					"expected at least one projection declaration",
				));
			}
			Ok(Self::Projections { span, projections })
		} else {
			return Err(syn::Error::new(
				span,
				format!("unknown projected option `{name}`; expected \
                 `module`, `projection_derives`, or `projections`"),
			));
		}
	}
}
