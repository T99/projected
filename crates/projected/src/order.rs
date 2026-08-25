use crate::ProjectedModel;

/// A model whose generated query metadata supports ordering.
///
/// Implementations are emitted by `#[projected(orderable)]` for source models
/// and by requesting `projected::Orderable` in `projection_derives(...)` for
/// generated projections.
pub trait Orderable: ProjectedModel {
	/// The Serde-enabled generated enum accepted as an ordering field.
	type OrderingField: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug + Clone;

	/// Maps an ordering input to the model's backend-neutral field identity.
	fn projected_field(field: Self::OrderingField) -> Self::Field;
}

/// Direction applied to an [`OrderBy`] expression.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum OrderingDirection {
	/// Sort from lower values to higher values.
	#[default]
	#[serde(alias = "asc")]
	Ascending,
	/// Sort from higher values to lower values.
	#[serde(alias = "desc")]
	Descending,
}

/// A backend-neutral request to order a model by one logical field.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct OrderBy<T: Orderable> {
	/// The generated field selected for ordering.
	pub field: T::OrderingField,
	/// The direction in which the selected field should be ordered.
	pub direction: OrderingDirection,
}
