//! Optional SeaORM integration for `projected` models.

#![warn(missing_docs)]

/// A projection that can be converted to its SeaORM active model.
pub trait SeaOrmProjection: projected::Projection {
	/// The entity-specific active model produced by this projection.
	type ActiveModel: sea_orm::ActiveModelTrait;

	/// Converts selected projection values into a SeaORM active model.
	///
	/// Required projection fields become `ActiveValue::Set`. An outer `None` on
	/// a projection-optional field becomes `ActiveValue::NotSet`; for nullable
	/// source fields, `Some(None)` remains an explicit `ActiveValue::Set(None)`.
	fn into_active_model(self) -> Self::ActiveModel;
}

#[cfg(test)]
mod tests {
	use projected::{ProjectedField, ProjectedModel, projected};
	use sea_orm::ActiveValue::{NotSet, Set};
	use sea_orm::entity::prelude::*;

	mod child {
		use super::*;

		#[sea_orm::model]
		#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
		#[sea_orm(table_name = "projected_child")]
		pub struct Model {
			#[sea_orm(primary_key, auto_increment = false)]
			pub id: i32,
			pub parent_id: i32,
			#[sea_orm(belongs_to, from = "parent_id", to = "id")]
			pub parent: BelongsTo<super::dense::Entity>,
		}

		impl ActiveModelBehavior for ActiveModel {}
	}

	mod dense {
		use super::*;

		#[projected(module, projections(ApiRecord(exclude(id), optional(name, note))))]
		#[sea_orm::model]
		#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
		#[sea_orm(table_name = "projected_dense")]
		pub struct Model {
			#[sea_orm(primary_key, auto_increment = false)]
			pub id: i32,
			pub name: String,
			pub note: Option<String>,
			#[sea_orm(has_many)]
			pub children: HasMany<super::child::Entity>,
		}

		impl ActiveModelBehavior for ActiveModel {}
	}

	#[test]
	fn dense_model_emits_scalar_metadata_and_projection_once() {
		fn assert_safe_derives<T: Clone + std::fmt::Debug + PartialEq + Eq>() {}
		fn accepts_model_ex(_: dense::ModelEx) {}

		assert_safe_derives::<dense::projection::ApiRecord>();
		let fields = dense::Model::fields();
		assert_eq!(fields.len(), 3);
		assert_eq!(dense::ModelField::Name.metadata().rust_name, "name");
		let _ = accepts_model_ex;
	}

	#[test]
	fn sea_orm_model_completion_is_preserved() {
		use dense::projection::{ApiRecord, ApiRecordMissing};

		let projection = ApiRecord {
			name: None,
			note: Some(None),
		};
		assert_eq!(
			projection.complete_with(7, "fallback".to_owned(), Some("unused".to_owned())),
			dense::Model {
				id: 7,
				name: "fallback".to_owned(),
				note: None,
			}
		);
		assert_eq!(
			ApiRecord {
				name: Some("model".to_owned()),
				note: None,
			}
			.to_model(ApiRecordMissing {
				id: 8,
				name: "unused".to_owned(),
				note: Some("fallback".to_owned()),
			}),
			dense::Model {
				id: 8,
				name: "model".to_owned(),
				note: Some("fallback".to_owned()),
			}
		);
	}

	#[test]
	fn active_model_preserves_not_set_and_nested_option_semantics() {
		use dense::projection::ApiRecord;

		let omitted = ApiRecord {
			name: None,
			note: None,
		}
		.to_active_model();
		assert_eq!(omitted.id, NotSet);
		assert_eq!(omitted.name, NotSet);
		assert_eq!(omitted.note, NotSet);

		let explicit_null = ApiRecord {
			name: Some("set".to_owned()),
			note: Some(None),
		}
		.to_active_model();
		assert_eq!(explicit_null.id, NotSet);
		assert_eq!(explicit_null.name, Set("set".to_owned()));
		assert_eq!(explicit_null.note, Set(None));

		let explicit_value = ApiRecord {
			name: Some("set".to_owned()),
			note: Some(Some("value".to_owned())),
		}
		.to_active_model();
		assert_eq!(explicit_value.note, Set(Some("value".to_owned())));
	}
}
