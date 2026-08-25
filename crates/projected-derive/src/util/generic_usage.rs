use std::collections::BTreeSet;

use syn::visit::{self, Visit};
use syn::{ExprPath, Generics, Lifetime, Type, TypePath};

/// Reports whether the supplied field types reference every declared generic
/// lifetime, type parameter, and const parameter.
///
/// Projection generation uses this result to decide whether it must emit a
/// hidden `PhantomData` marker. Only syntactic references are required; bounds
/// and where clauses do not make a field carry a generic parameter.
pub fn uses_all_generic_parameters<'a>(
	generics: &Generics,
	types: impl Iterator<Item = &'a Type>,
) -> bool {
	let expected = generics
		.params
		.iter()
		.map(|parameter| match parameter {
			syn::GenericParam::Lifetime(parameter) => parameter.lifetime.ident.to_string(),
			syn::GenericParam::Type(parameter) => parameter.ident.to_string(),
			syn::GenericParam::Const(parameter) => parameter.ident.to_string(),
		})
		.collect::<BTreeSet<_>>();
	let mut visitor = GenericUseVisitor {
		expected: &expected,
		used: BTreeSet::new(),
	};
	for ty in types {
		visitor.visit_type(ty);
	}
	visitor.used == expected
}

/// AST visitor that records generic parameter names used by field types.
struct GenericUseVisitor<'a> {
	/// Declared generic names whose use is relevant.
	expected: &'a BTreeSet<String>,
	/// Relevant generic names encountered during traversal.
	used: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for GenericUseVisitor<'_> {
	/// Records declared lifetimes while continuing traversal into nested syntax.
	fn visit_lifetime(&mut self, lifetime: &'ast Lifetime) {
		let name = lifetime.ident.to_string();
		if self.expected.contains(&name) {
			self.used.insert(name);
		}
		visit::visit_lifetime(self, lifetime);
	}

	/// Records the leading segment of unqualified type paths such as `T` or
	/// `T::Associated`, then visits nested generic arguments.
	fn visit_type_path(&mut self, path: &'ast TypePath) {
		if path.qself.is_none()
			&& let Some(segment) = path.path.segments.first()
		{
			let name = segment.ident.to_string();
			if self.expected.contains(&name) {
				self.used.insert(name);
			}
		}
		visit::visit_type_path(self, path);
	}

	/// Records const-generic identifiers used as expression paths in types.
	fn visit_expr_path(&mut self, path: &'ast ExprPath) {
		if path.qself.is_none()
			&& let Some(segment) = path.path.segments.first()
		{
			let name = segment.ident.to_string();
			if self.expected.contains(&name) {
				self.used.insert(name);
			}
		}
		visit::visit_expr_path(self, path);
	}
}
