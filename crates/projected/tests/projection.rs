use projected::{ProjectedField, ProjectedFieldMapping, ProjectedModel, Projection, projected, __Projected};

#[projected(projections(
	IncludeOnly(include(name, nullable), optional(nullable)),
	ExcludeOnly(exclude(id), optional(name)),
	Bare,
	AllOptional(optional)
))]
#[derive(Debug, PartialEq)]
struct Example {
	id: u32,
	name: String,
	nullable: Option<i32>,
	count: i32,
}

#[test]
fn selection_and_optional_types_are_exact() {
	let projection = IncludeOnly {
		name: "included".to_owned(),
		nullable: Some(Some(7)),
	};
	let _: Option<Option<i32>> = projection.nullable;
	let completed = projection.complete_with(11, None, 3);
	assert_eq!(
		completed,
		Example {
			id: 11,
			name: "included".to_owned(),
			nullable: Some(7),
			count: 3,
		}
	);
}

#[test]
fn completion_uses_fallback_only_when_outer_option_is_none() {
	let supplied = ExcludeOnly {
		name: Some("projected".to_owned()),
		nullable: None,
		count: 8,
	};
	assert_eq!(
		supplied.complete_with(1, "fallback".to_owned()),
		Example {
			id: 1,
			name: "projected".to_owned(),
			nullable: None,
			count: 8,
		}
	);

	let omitted = ExcludeOnly {
		name: None,
		nullable: Some(2),
		count: 9,
	};
	assert_eq!(
		omitted.complete_with(2, "fallback".to_owned()),
		Example {
			id: 2,
			name: "fallback".to_owned(),
			nullable: Some(2),
			count: 9,
		}
	);
}

#[test]
fn lossless_projection_round_trips_through_from() {
	let base = Example {
		id: 42,
		name: "athena".to_owned(),
		nullable: Some(5),
		count: 17,
	};
	let projection = Bare::from(base);
	let reconstructed = Example::from(projection);
	assert_eq!(
		reconstructed,
		Example {
			id: 42,
			name: "athena".to_owned(),
			nullable: Some(5),
			count: 17,
		}
	);
}

#[test]
fn base_to_optional_projection_wraps_every_included_field() {
	let projection = AllOptional::from(Example {
		id: 2,
		name: "wrapped".to_owned(),
		nullable: None,
		count: 4,
	});
	assert_eq!(projection.id, Some(2));
	assert_eq!(projection.name.as_deref(), Some("wrapped"));
	assert_eq!(projection.nullable, Some(None));
	assert_eq!(projection.count, Some(4));
	let reconstructed = Projection::complete(
		projection,
		AllOptionalMissing {
			id: 0,
			name: "unused".to_owned(),
			nullable: Some(99),
			count: 0,
		},
	);
	assert_eq!(reconstructed.nullable, None);
}

#[projected(projections(First(include(alpha)), Second))]
#[derive(Debug, PartialEq)]
struct FieldConfigured {
	alpha: i32,
	#[projected(include(First), exclude(Second))]
	beta: String,
	#[projected(optional)]
	gamma: bool,
}

#[test]
fn field_level_rules_apply_to_named_or_all_projections() {
	let first = First {
		alpha: 3,
		beta: "field".to_owned(),
		gamma: None,
	};
	assert_eq!(
		first.complete_with(false),
		FieldConfigured {
			alpha: 3,
			beta: "field".to_owned(),
			gamma: false,
		}
	);
	let second = Second {
		alpha: 5,
		gamma: Some(true),
	};
	assert_eq!(
		second.complete_with("missing".to_owned(), false),
		FieldConfigured {
			alpha: 5,
			beta: "missing".to_owned(),
			gamma: true,
		}
	);
}

#[projected(projections(GenericProjection))]
struct Generic<'a, T, const N: usize>
where
	T: PartialEq,
{
	borrowed: &'a T,
	values: [u8; N],
}

#[test]
fn generics_lifetimes_const_generics_and_where_clauses_are_preserved() {
	let value = 12;
	let projection = GenericProjection::from(Generic {
		borrowed: &value,
		values: [1, 2, 3],
	});
	let base = projection.into_base();
	assert_eq!(*base.borrowed, 12);
	assert_eq!(base.values, [1, 2, 3]);
}

#[projected(projections(ConditionalProjection))]
struct Conditional<T> {
	value: T,
	#[cfg(any())]
	conditional: String,
}

#[test]
fn cfg_fields_stay_consistent_across_generated_items() {
	let projection = ConditionalProjection::from(Conditional { value: 4_u8 });
	assert_eq!(projection.into_base().value, 4);
}

#[projected(
	projections(Serialized(exclude(internal_id))),
	projection_derives(serde::Serialize),
	propagate(serde)
)]
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SerializedSource {
	internal_id: i32,
	#[serde(rename = "displayName")]
	display_name: String,
}

#[test]
fn configured_derives_and_safe_attributes_are_propagated() {
	let value = serde_json::to_value(Serialized {
		display_name: "Athena".to_owned(),
	})
	.expect("generated projection serializes");
	assert_eq!(value, serde_json::json!({ "displayName": "Athena" }));
}

mod default_module_fixture {
	use super::*;

	pub type DisplayName = String;

	#[projected(
		module,
		projections(Public(exclude(secret)), Summary(include(id, name)))
	)]
	#[derive(Debug, PartialEq)]
	pub struct NestedModel {
		pub id: i32,
		pub name: DisplayName,
		pub secret: String,
	}
}

#[test]
fn default_module_contains_multiple_projections_and_existing_apis() {
	use default_module_fixture::NestedModel;
	use default_module_fixture::projection::{Public, Summary};

	let public = Public::from(NestedModel {
		id: 1,
		name: "Athena".to_owned(),
		secret: "hash".to_owned(),
	});
	assert_eq!(
		public.complete_with("hash".to_owned()),
		NestedModel {
			id: 1,
			name: "Athena".to_owned(),
			secret: "hash".to_owned(),
		}
	);

	let summary = Summary {
		id: 2,
		name: "Summary".to_owned(),
	};
	assert_eq!(
		summary.complete_with("omitted".to_owned()),
		NestedModel {
			id: 2,
			name: "Summary".to_owned(),
			secret: "omitted".to_owned(),
		}
	);
}

mod custom_module_fixture {
	use super::*;

	#[projected(module = views, projections(CrateVisible))]
	pub(crate) struct CrateModel {
		pub value: i32,
	}
}

#[test]
fn custom_module_inherits_crate_visibility() {
	let projection =
		custom_module_fixture::views::CrateVisible::from(custom_module_fixture::CrateModel {
			value: 9,
		});
	assert_eq!(projection.into_base().value, 9);
}

#[test]
fn generated_metadata_is_shared_and_maps_projection_fields_to_source_fields() {
	assert_eq!(Example::fields().len(), 4);
	assert_eq!(ExampleField::Name.metadata().rust_name, "name");
	assert_eq!(ExampleField::Name.metadata().serialize_name, "name");
	assert_eq!(
		<IncludeOnlyField as ProjectedFieldMapping<Example>>::source_field(
			IncludeOnlyField::Nullable
		),
		ExampleField::Nullable
	);
}
