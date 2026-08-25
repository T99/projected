use projected::projected;
use sea_orm::entity::prelude::*;

#[projected(
	module,
	projections(ApiModel(exclude(id), optional(note)))
)]
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "projected_compile_fixture")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	pub id: i32,
	pub name: String,
	pub note: Option<String>,
}

impl ActiveModelBehavior for ActiveModel {}

fn main() {
	let active: ActiveModel = projection::ApiModel {
		name: "fixture".to_owned(),
		note: Some(None),
	}
	.to_active_model();
	assert!(active.id.is_not_set());
	let _ = projection::ApiModel {
		name: "fixture".to_owned(),
		note: None,
	}
	.complete_with(1, Some("fallback".to_owned()));
}
