use syn::punctuated::Punctuated;
use syn::{Attribute, Expr, ExprLit, Lit, LitStr, Meta, Token};

/// A Serde setting that may differ between serialization and deserialization.
#[derive(Clone)]
struct Directional<T> {
	/// Serialization-side value, when explicitly configured.
	serialize: Option<T>,
	/// Deserialization-side value, when explicitly configured.
	deserialize: Option<T>,
}

impl<T> Default for Directional<T> {
	/// Creates an unresolved setting for both directions.
	fn default() -> Self {
		Self {
			serialize: None,
			deserialize: None,
		}
	}
}

impl<T: Clone> Directional<T> {
	/// Applies one non-directional setting to both directions.
	fn set_both(&mut self, value: T) {
		self.serialize = Some(value.clone());
		self.deserialize = Some(value);
	}
}

/// Supported Serde container rename conventions.
#[derive(Clone, Copy)]
enum RenameRule {
	/// Serde `lowercase`.
	Lower,
	/// Serde `UPPERCASE`.
	Upper,
	/// Serde `PascalCase`.
	Pascal,
	/// Serde `camelCase`.
	Camel,
	/// Serde `snake_case`.
	Snake,
	/// Serde `SCREAMING_SNAKE_CASE`.
	ScreamingSnake,
	/// Serde `kebab-case`.
	Kebab,
	/// Serde `SCREAMING-KEBAB-CASE`.
	ScreamingKebab,
}

impl RenameRule {
	/// Parses the exact spellings accepted by Serde's `rename_all` attribute.
	fn parse(value: &LitStr) -> syn::Result<Self> {
		match value.value().as_str() {
			"lowercase" => Ok(Self::Lower),
			"UPPERCASE" => Ok(Self::Upper),
			"PascalCase" => Ok(Self::Pascal),
			"camelCase" => Ok(Self::Camel),
			"snake_case" => Ok(Self::Snake),
			"SCREAMING_SNAKE_CASE" => Ok(Self::ScreamingSnake),
			"kebab-case" => Ok(Self::Kebab),
			"SCREAMING-KEBAB-CASE" => Ok(Self::ScreamingKebab),
			other => Err(syn::Error::new(
				value.span(),
				format!("unknown Serde rename rule `{other}`"),
			)),
		}
	}

	/// Applies this container rule to a Rust field identifier.
	///
	/// Field names arrive in snake case from Rust source. Lowercase and snake
	/// case are therefore identity operations, matching Serde's field behavior.
	fn apply_to_field(self, field: &str) -> String {
		match self {
			Self::Lower | Self::Snake => field.to_owned(),
			Self::Upper | Self::ScreamingSnake => field.to_ascii_uppercase(),
			Self::Pascal => pascal_case(field),
			Self::Camel => {
				let pascal = pascal_case(field);
				pascal[..1].to_ascii_lowercase() + &pascal[1..]
			}
			Self::Kebab => field.replace('_', "-"),
			Self::ScreamingKebab => field.to_ascii_uppercase().replace('_', "-"),
		}
	}
}

/// Converts a snake-case Rust field name to PascalCase without allocating
/// intermediate word collections.
fn pascal_case(field: &str) -> String {
	let mut pascal = String::with_capacity(field.len());
	let mut capitalize = true;
	for character in field.chars() {
		if character == '_' {
			capitalize = true;
		} else if capitalize {
			pascal.push(character.to_ascii_uppercase());
			capitalize = false;
		} else {
			pascal.push(character);
		}
	}
	pascal
}

/// Directional Serde-visible names resolved for one logical source field.
#[derive(Clone)]
pub struct FieldNames {
	/// Name emitted during serialization.
	pub serialize: String,
	/// Name accepted during deserialization.
	pub deserialize: String,
}

/// Resolves one field's Serde-visible names exactly once.
///
/// Field-level `rename` takes precedence over container-level `rename_all` for
/// each direction independently. If neither is present, the normalized Rust
/// identifier is returned for both directions.
pub fn resolve(
	container_attrs: &[Attribute],
	field_attrs: &[Attribute],
	field_name: &str,
) -> syn::Result<FieldNames> {
	let rename_all = parse_directional(container_attrs, "rename_all", RenameRule::parse)?;
	let rename = parse_directional(field_attrs, "rename", |value| Ok(value.value()))?;

	let serialize = rename.serialize.unwrap_or_else(|| {
		rename_all.serialize.map_or_else(
			|| field_name.to_owned(),
			|rule| rule.apply_to_field(field_name),
		)
	});
	let deserialize = rename.deserialize.unwrap_or_else(|| {
		rename_all.deserialize.map_or_else(
			|| field_name.to_owned(),
			|rule| rule.apply_to_field(field_name),
		)
	});

	Ok(FieldNames {
		serialize,
		deserialize,
	})
}

/// Parses either `name = "..."` or directional
/// `name(serialize = "...", deserialize = "...")` Serde metadata.
///
/// Later matching attributes overwrite earlier values, mirroring the existing
/// Athena parser. `parse_value` lets the same machinery resolve literal field
/// names and validated container rename rules.
fn parse_directional<T: Clone>(
	attrs: &[Attribute],
	name: &str,
	parse_value: impl Fn(&LitStr) -> syn::Result<T>,
) -> syn::Result<Directional<T>> {
	let mut result = Directional::default();
	for attr in attrs.iter().filter(|attr| attr.path().is_ident("serde")) {
		let metas = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
		for meta in metas {
			if !meta.path().is_ident(name) {
				continue;
			}
			match meta {
				Meta::NameValue(meta) => result.set_both(parse_value(lit_str(&meta.value)?)?),
				Meta::List(meta) => {
					let directions =
						meta.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
					for direction in directions {
						let Meta::NameValue(direction) = direction else {
							return Err(syn::Error::new_spanned(
								direction,
								"expected `serialize = \"...\"` or `deserialize = \"...\"`",
							));
						};
						let value = parse_value(lit_str(&direction.value)?)?;
						if direction.path.is_ident("serialize") {
							result.serialize = Some(value);
						} else if direction.path.is_ident("deserialize") {
							result.deserialize = Some(value);
						} else {
							return Err(syn::Error::new_spanned(
								direction.path,
								"expected `serialize` or `deserialize`",
							));
						}
					}
				}
				Meta::Path(path) => {
					return Err(syn::Error::new_spanned(
						path,
						format!("expected `{name} = \"...\"`"),
					));
				}
			}
		}
	}
	Ok(result)
}

/// Extracts a string literal from a Serde name-value expression.
fn lit_str(expression: &Expr) -> syn::Result<&LitStr> {
	if let Expr::Lit(ExprLit {
		lit: Lit::Str(value),
		..
	}) = expression
	{
		Ok(value)
	} else {
		Err(syn::Error::new_spanned(
			expression,
			"expected a string literal",
		))
	}
}
