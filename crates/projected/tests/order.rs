#![allow(dead_code)]

use projected::{Orderable, ProjectedField, ProjectedModel, projected, __Projected};

#[projected(orderable)]
#[derive(serde::Serialize, serde::Deserialize)]
struct DefaultNames {
	foo_bar: String,
}

#[test]
fn defaults_to_the_rust_field_name() {
	assert_eq!(
		serde_json::to_string(&DefaultNamesOrderField::FooBar).unwrap(),
		r#""foo_bar""#
	);
	assert_eq!(
		serde_json::from_str::<DefaultNamesOrderField>(r#""foo_bar""#).unwrap(),
		DefaultNamesOrderField::FooBar
	);
}

#[projected(orderable)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CamelCaseNames {
	foo_bar: String,
}

#[test]
fn applies_container_rename_all() {
	assert_eq!(
		serde_json::to_string(&CamelCaseNamesOrderField::FooBar).unwrap(),
		r#""fooBar""#
	);
	assert_eq!(
		serde_json::from_str::<CamelCaseNamesOrderField>(r#""fooBar""#).unwrap(),
		CamelCaseNamesOrderField::FooBar
	);
}

#[projected(orderable)]
#[derive(serde::Serialize, serde::Deserialize)]
struct RenamedField {
	#[serde(rename = "custom")]
	foo_bar: String,
}

#[test]
fn applies_field_rename() {
	assert_eq!(
		serde_json::from_str::<RenamedFieldOrderField>(r#""custom""#).unwrap(),
		RenamedFieldOrderField::FooBar
	);
}

#[projected(orderable)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RenameOverridesContainer {
	#[serde(rename = "custom")]
	foo_bar: String,
}

#[test]
fn field_rename_overrides_container_rename_all() {
	assert_eq!(
		serde_json::to_string(&RenameOverridesContainerOrderField::FooBar).unwrap(),
		r#""custom""#
	);
	assert!(serde_json::from_str::<RenameOverridesContainerOrderField>(r#""fooBar""#).is_err());
}

#[projected(orderable)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "SCREAMING_SNAKE_CASE"))]
struct DirectionalNames {
	foo_bar: String,
	#[serde(rename(serialize = "output", deserialize = "input"))]
	renamed: String,
}

#[test]
fn applies_directional_container_and_field_names() {
	assert_eq!(
		serde_json::to_string(&DirectionalNamesOrderField::FooBar).unwrap(),
		r#""fooBar""#
	);
	assert_eq!(
		serde_json::from_str::<DirectionalNamesOrderField>(r#""FOO_BAR""#).unwrap(),
		DirectionalNamesOrderField::FooBar
	);
	assert_eq!(
		serde_json::to_string(&DirectionalNamesOrderField::Renamed).unwrap(),
		r#""output""#
	);
	assert_eq!(
		serde_json::from_str::<DirectionalNamesOrderField>(r#""input""#).unwrap(),
		DirectionalNamesOrderField::Renamed
	);
}

#[projected(orderable)]
struct SkippedField {
	visible: String,
	#[projected(order(skip))]
	hidden: String,
}

#[test]
fn order_skip_uses_the_projected_field_configuration() {
	let fields = SkippedField::fields();
	assert_eq!(fields.len(), 2);
	assert!(SkippedFieldField::Visible.metadata().orderable);
	assert!(!SkippedFieldField::Hidden.metadata().orderable);
	assert_eq!(
		SkippedField::projected_field(SkippedFieldOrderField::Visible),
		SkippedFieldField::Visible
	);
}

#[projected(
	projections(TeamProjection),
	projection_derives(serde::Serialize, serde::Deserialize, projected::Orderable),
	propagate(serde)
)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct TeamSource {
	team_number: i32,
}

#[test]
fn projected_ordering_reuses_shared_serde_names() {
	assert_eq!(
		serde_json::from_str::<TeamProjectionOrderField>(r#""teamNumber""#).unwrap(),
		TeamProjectionOrderField::TeamNumber
	);
	assert_eq!(
		TeamProjection::projected_field(TeamProjectionOrderField::TeamNumber),
		TeamProjectionField::TeamNumber
	);
	assert_eq!(
		TeamProjectionField::TeamNumber.metadata().serialize_name,
		"teamNumber"
	);
}
