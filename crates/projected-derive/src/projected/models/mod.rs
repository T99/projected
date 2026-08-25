/// Safe derive inheritance and source attribute propagation.
mod attribute_policy;
/// Shared include/exclude/optional action vocabulary.
mod field_action;
/// Struct-level projection declaration parsing and resolution.
mod projection_declaration;
/// Field-level configuration parsing.
mod raw_field_rule;
/// Struct-level modifier parsing.
mod raw_projection_modifier;
/// Per-field resolved projection state.
mod resolved_field;
/// Final model-level emission.
mod resolved_model;
/// Final projection-level emission.
mod resolved_projection;
/// Explicit-versus-implicit rule provenance.
mod rule_origin;
/// Post-transformation source parsing and cross-projection validation.
mod source_model;

pub use attribute_policy::AttributePolicy;
pub use field_action::FieldAction;
pub use projection_declaration::ProjectionDeclaration;
pub use raw_field_rule::RawFieldConfiguration;
pub use raw_projection_modifier::RawProjectionModifier;
pub use resolved_field::ResolvedField;
pub use resolved_model::{ResolvedModel, SeaOrmPaths};
pub use resolved_projection::ResolvedProjection;
pub use rule_origin::RuleOrigin;
pub use source_model::SourceModel;

use crate::projected::serde_name::FieldNames;
use proc_macro2::{Ident, Span};
use syn::{Attribute, Path, Type};

/// Attribute categories that may be copied from a source item to generated
/// projection items.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PropagatedAttribute {
	/// Rust documentation attributes.
	Doc,
	/// Direct conditional-compilation attributes.
	Cfg,
	/// Conditional attribute containers after nested filtering.
	CfgAttr,
	/// Serde helper attributes.
	Serde,
	/// Schema helper attributes from `schema` or `schemars`.
	Schema,
}

/// Struct-wide generation options resolved from `#[projected(...)]`.
#[derive(Default)]
pub struct ProjectedOptions {
	/// Optional module receiving generated projection types.
	pub module: Option<Ident>,
	/// Whether the source model receives an `Orderable` implementation.
	pub orderable: bool,
}

/// Role assigned to a post-transformation SeaORM struct.
pub struct SeaOrmInfo {
	/// Whether scalar SeaORM integration should be emitted for this struct.
	pub is_model: bool,
	/// Whether this is a generated relationship-bearing companion whose generic
	/// output must be suppressed to avoid duplicate definitions.
	pub is_generated_companion: bool,
}

/// Parsed source field plus all backend-neutral metadata resolved once for it.
pub struct SourceField {
	/// Rust field identifier.
	pub ident: Ident,
	/// Rust field type exactly as declared after transforming attributes.
	pub ty: Type,
	/// Original field attributes retained for controlled propagation and `cfg`.
	pub attrs: Vec<Attribute>,
	/// Directional Serde-visible names.
	pub names: FieldNames,
	/// Whether ordering may expose this field.
	pub orderable: bool,
	/// Projection-selection rules declared directly on this field.
	pub rules: Vec<FieldRule>,
}

/// Normalized projection modifier declared at struct level.
pub enum ProjectionModifier {
	/// Includes the named source fields.
	Include {
		/// Modifier span used for diagnostics.
		span: Span,
		/// Referenced source fields.
		fields: Vec<Ident>,
	},
	/// Excludes the named source fields.
	Exclude {
		/// Modifier span used for diagnostics.
		span: Span,
		/// Referenced source fields.
		fields: Vec<Ident>,
	},
	/// Makes every currently included field projection-optional.
	OptionalAll {
		/// Modifier span used for diagnostics.
		span: Span,
	},
	/// Includes the named fields when possible and makes them projection-optional.
	OptionalFields {
		/// Modifier span used for diagnostics.
		span: Span,
		/// Referenced source fields.
		fields: Vec<Ident>,
	},
}

/// One normalized field-level rule and its target projections.
pub struct FieldRule {
	/// Selection or optionality change requested by the rule.
	pub action: FieldAction,
	/// Declared projections affected by the rule.
	pub targets: FieldTargets,
	/// Source span used for conflict diagnostics.
	pub span: Span,
}

/// Projection targets for a field-level rule.
pub enum FieldTargets {
	/// Every declared projection.
	All,
	/// Only the explicitly named projections.
	Named(Vec<Ident>),
}

/// Whether a source field appears in a particular projection.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Selection {
	/// The projection owns a value for the field.
	Included,
	/// Completion obtains the field from its missing-values input.
	Excluded,
}

/// Returns whether a configured projection derive path requests orchestrated
/// ordering generation rather than a literal derive invocation.
pub fn is_orderable_derive(path: &Path) -> bool {
	path.segments
		.last()
		.is_some_and(|segment| segment.ident == "Orderable")
}
