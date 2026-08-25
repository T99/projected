use crate::projected::models::{PropagatedAttribute, is_orderable_derive};
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::{Attribute, Meta, Path, Token};

/// Derives whose semantics can be inherited safely by owned projection values.
const SAFE_PROJECTION_DERIVES: &[&str] = &[
	"Debug",
	"Clone",
	"Copy",
	"PartialEq",
	"Eq",
	"PartialOrd",
	"Ord",
	"Hash",
	"Default",
];

/// Controls derive inheritance and which source attributes may be propagated to
/// generated projection and missing-values types.
pub struct AttributePolicy {
	/// Enabled propagation categories.
	pub allowed: Vec<PropagatedAttribute>,
	/// Literal derives emitted on generated projection support types.
	pub projection_derives: Vec<Path>,
	/// Whether projections receive orchestrated ordering generation.
	pub projections_orderable: bool,
}

impl AttributePolicy {
	/// Extracts the conservative derive allowlist that can safely be inherited by
	/// owned projection structs.
	///
	/// Paths are de-duplicated while preserving source order. Only the final path
	/// segment is used for allowlist membership so qualified safe derives work.
	pub fn safe_derives(attrs: &[Attribute]) -> syn::Result<Vec<Path>> {
		let mut derives = Vec::new();
		for attr in attrs {
			if !attr.path().is_ident("derive") {
				continue;
			}
			let paths = attr.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)?;
			for path in paths {
				let Some(ident) = path.segments.last().map(|segment| &segment.ident) else {
					continue;
				};
				if SAFE_PROJECTION_DERIVES
					.iter()
					.any(|allowed| ident == allowed)
					&& !derives
						.iter()
						.any(|existing| Self::same_derive(existing, &path))
				{
					derives.push(path);
				}
			}
		}
		Ok(derives)
	}

	/// Returns the subset of `attrs` permitted by this policy.
	///
	/// `cfg_attr` receives nested filtering instead of being copied wholesale,
	/// preventing control or backend helper attributes from leaking into output.
	pub fn propagated(&self, attrs: &[Attribute]) -> Vec<Attribute> {
		attrs
			.iter()
			.filter_map(|attr| self.propagate_one(attr))
			.collect()
	}

	/// Applies propagation policy to one source attribute.
	fn propagate_one(&self, attr: &Attribute) -> Option<Attribute> {
		if attr.path().is_ident("cfg_attr") {
			return self
				.allowed
				.contains(&PropagatedAttribute::CfgAttr)
				.then(|| {
					crate::projected::attributes::filtered_cfg_attr(attr, |meta| {
						self.meta_is_allowed(meta)
					})
				})
				.flatten();
		}
		self.meta_is_allowed(&attr.meta).then(|| attr.clone())
	}

	/// Tests whether a single metadata path belongs to an enabled category.
	fn meta_is_allowed(&self, meta: &Meta) -> bool {
		let path = meta.path();
		self.allowed.iter().any(|allowed| match allowed {
			PropagatedAttribute::Doc => path.is_ident("doc"),
			PropagatedAttribute::Cfg => path.is_ident("cfg"),
			PropagatedAttribute::CfgAttr => path.is_ident("cfg_attr"),
			PropagatedAttribute::Serde => path.is_ident("serde"),
			PropagatedAttribute::Schema => path.is_ident("schema") || path.is_ident("schemars"),
		})
	}

	/// Adds safe derives found in one source `derive` attribute.
	pub fn inherit_safe_derives(&mut self, attr: &Attribute) -> syn::Result<()> {
		for derive in Self::safe_derives(std::slice::from_ref(attr))? {
			self.push_projection_derive(derive);
		}
		Ok(())
	}

	/// Adds an explicit projection derive unless it is already present.
	///
	/// A path ending in `Orderable` is intercepted as an orchestration request;
	/// it is never emitted as an independent derive that would reparse metadata.
	pub fn push_projection_derive(&mut self, derive: Path) {
		if is_orderable_derive(&derive) {
			self.projections_orderable = true;
			return;
		}
		if self
			.projection_derives
			.iter()
			.any(|existing| Self::same_derive(existing, &derive))
		{
			return;
		}
		self.projection_derives.push(derive);
	}

	/// Compares derive paths using semantic last-segment equality for the safe
	/// allowlist and token equality for arbitrary additional derives.
	fn same_derive(left: &Path, right: &Path) -> bool {
		let left_name = left.segments.last().map(|segment| &segment.ident);
		let right_name = right.segments.last().map(|segment| &segment.ident);
		if let (Some(left_name), Some(right_name)) = (left_name, right_name)
			&& left_name == right_name
			&& SAFE_PROJECTION_DERIVES.iter().any(|safe| left_name == safe)
		{
			return true;
		}
		match (left.get_ident(), right.get_ident()) {
			(Some(left), Some(right)) => left == right,
			_ => left.to_token_stream().to_string() == right.to_token_stream().to_string(),
		}
	}
}

impl Default for AttributePolicy {
	/// Creates the compatibility policy: documentation and conditional attributes
	/// propagate by default, while Serde and schema helpers require opt-in.
	fn default() -> Self {
		Self {
			allowed: vec![
				PropagatedAttribute::Doc,
				PropagatedAttribute::Cfg,
				PropagatedAttribute::CfgAttr,
			],
			projection_derives: Vec::new(),
			projections_orderable: false,
		}
	}
}
