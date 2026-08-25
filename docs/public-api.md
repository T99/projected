# Intended public API

This is the downstream-facing API implemented by the initial extraction. `Projectable`,
`projectable_model`, `#[projectable]`, and `#[projection]` are intentionally absent.

## Imports and crate setup

The `projected` runtime crate re-exports the public attribute:

```rust,ignore
use projected::{projected, OrderBy, Orderable, OrderingDirection, Projection};
```

SeaORM users also depend directly on `projected-seaorm`, `sea-orm`, and `projected`. Generated
inherent methods require no extension-trait import; `projected_seaorm::SeaOrmProjection` is the
runtime integration trait. Feature-gated distribution remains an open packaging question.

## Plain struct

Projection declarations move inside the one struct-level attribute. Existing modifier semantics
and source field order are retained.

```rust,ignore
use projected::{projected, Projection};

#[projected(
	projections(
		Public(exclude(password_hash), optional(display_name))
	)
)]
#[derive(Debug, PartialEq)]
pub struct User {
	pub id: i64,
	pub display_name: String,
	pub password_hash: String,
}

let public = Public {
	id: 7,
	display_name: None,
};

let user = public.complete_with(
	"secret hash".to_owned(),
	"fallback name".to_owned(),
);
```

The completion input can be named generically as `<Public as Projection>::Missing`; the
concrete helper is `PublicMissingFields` when values are required. A lossless projection uses
`Missing = ()`, supports `into_base()`, and implements `From<Projection> for Base`.

Projection-optional values retain Athena's nested-option semantics. If a source field is
`Option<String>`, its optional projection type is `Option<Option<String>>`: `None` means omitted,
`Some(None)` means explicitly null, and `Some(Some(value))` means explicitly set.

## Multiple projections and module placement

```rust,ignore
use projected::projected;

#[projected(
	module,
	projections(
		Public(exclude(password_hash, internal_note)),
		Summary(include(id, display_name)),
		Patch(include(display_name, email), optional)
	)
)]
pub struct User {
	pub id: i64,
	pub display_name: String,
	pub email: String,
	pub password_hash: String,
	pub internal_note: String,
}

let _: projection::Public;
let _: projection::Summary;
let _: projection::Patch;
```

Bare `module` emits a `projection` module. `module = views` selects another Rust identifier. The
module, generated structs, support types, and their public fields copy the source visibility. As in
Athena, generated modules do not merge across separate model invocations.

Declarations have the current meanings:

- `Name` includes every field and makes each required;
- `Name(include(a, b))` starts excluded and selects only named fields;
- `Name(exclude(a, b))` starts included and removes named fields;
- `optional` makes all currently selected fields projection-optional;
- `optional(a, b)` includes the named fields when not explicitly excluded and makes them optional.

Combining `include` and `exclude`, naming an unknown field, duplicating an explicit rule, or
reusing a projection name is a compile error.

## Field-level `#[projected]` rules

Field rules use the same public attribute name. Parenthesized names target declared projections;
an omitted target list applies to all projections.

```rust,ignore
use projected::projected;

#[projected(projections(Public, Summary(include(id))))]
struct User {
	id: i64,

	#[projected(exclude(Public), include(Summary))]
	internal_name: String,

	#[projected(optional(Public, Summary))]
	display_name: String,

	#[projected(exclude)]
	password_hash: String,
}
```

Field-level rules may adjust declared projections but cannot declare new ones. `optional` includes
an implicitly excluded field when necessary, but conflicts with an explicit exclusion.

Ordering-specific field controls also belong under this attribute. The replacement for the current
`#[orderable(skip)]` is `#[projected(order(skip))]`; there is no second field annotation namespace.

## Safe derive inheritance

Safe derives on the source are inherited automatically because `#[projected]` runs before any
transforming attribute and records the original derive list.

```rust,ignore
use projected::projected;

#[projected(projections(Public(exclude(secret))))]
#[derive(Clone, Debug, PartialEq, Eq)]
struct Model {
	id: i32,
	secret: String,
}

fn needs_safe_derives<T: Clone + core::fmt::Debug + PartialEq + Eq>() {}
needs_safe_derives::<Public>();
```

The initial safe allowlist matches Athena: `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`,
`PartialOrd`, `Ord`, `Hash`, and `Default`. A derive is inherited only when it is present on the
source and is valid for the generated fields. `#[projected]` replaces the current need for a
separate `#[projectable_model]` attribute.

## Explicit additional projection derives

Derives outside the safe allowlist remain explicit and apply to projection structs and generated
missing-fields structs:

```rust,ignore
use projected::projected;

#[projected(
	projections(Public(exclude(secret))),
	projection_derives(
		serde::Serialize,
		serde::Deserialize,
		utoipa::ToSchema
	),
	propagate(serde, schema)
)]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Model {
	id: i32,
	display_name: String,
	secret: String,
}
```

`projection_derives(...)` composes with inherited safe derives and de-duplicates equivalent safe
paths. `doc`, `cfg`, and `cfg_attr` propagation remains enabled by default. `propagate(serde)` opts
in Serde attributes; `propagate(schema)` covers `schema` and `schemars` attributes.

Control attributes (`projected`, internal helper attributes, and backend helper attributes) are
never copied blindly to generated projections. Nested `cfg_attr` content is filtered by the same
policy.

## Serde propagation and shared wire names

Serde-visible names are resolved once for generated field metadata and reused by ordering and
future query features.

```rust,ignore
use projected::projected;

#[projected(
	projections(Public(exclude(internal_id))),
	projection_derives(serde::Serialize, serde::Deserialize),
	propagate(serde)
)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Model {
	internal_id: i32,

	#[serde(rename = "name")]
	display_name: String,
}
```

Here the projection serializes `display_name` as `name`; its ordering metadata uses the same name.
Directional forms remain distinct:

```rust,ignore
#[serde(rename_all(serialize = "camelCase", deserialize = "SCREAMING_SNAKE_CASE"))]
struct Directional {
	#[serde(rename(serialize = "output", deserialize = "input"))]
	value: String,
}
```

Field `rename` overrides container `rename_all`. Behavior for Serde `alias`, `flatten`, and skip
forms requires explicit investigation before those are treated as query-field naming rules.

## Ordering

The runtime API retains Athena's shape:

```rust,ignore
pub trait Orderable: ProjectedModel {
	type OrderingField;
	fn projected_field(field: Self::OrderingField) -> Self::Field;
}

pub struct OrderBy<T: Orderable> {
	pub field: T::OrderingField,
	pub direction: OrderingDirection,
}
```

`OrderingDirection::{Ascending, Descending}` serialize in camelCase and accept `asc`/`desc`
aliases. Generated ordering fields remain serializable/deserializable and map to shared
`ProjectedField` identities.

`#[projected(orderable)]` generates ordering support for the source. To add it to every generated
projection, include `projected::Orderable` in `projection_derives(...)`; the orchestrator recognizes
that path as a generation request rather than emitting a second derive. `{Type}OrderField` is the
current generated enum name. Both paths consume the shared resolved metadata and do not reparse
Serde naming.

```rust,ignore
#[projected(
	orderable,
	projections(Public(exclude(secret))),
	projection_derives(projected::Orderable)
)]
struct Model {
	id: i32,
	#[projected(order(skip))]
	secret: String,
}
```

Future `Filterable` should follow the same pattern and is intentionally not implemented now.

## SeaORM

`#[projected]` must be placed before `#[sea_orm::model]` so the public macro can capture the
original derives and arrange generation against SeaORM's transformed scalar `Model`. When
possible, the macro should try to expose helpful error messages if the user misorders the attributes.

```rust,ignore
use projected::{projected, Projection};
use sea_orm::entity::prelude::*;

#[projected(
	module,
	projections(ApiModel(exclude(id), optional(note))),
	projection_derives(serde::Serialize, serde::Deserialize),
	propagate(serde)
)]
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "projected_fixture")]
#[serde(rename_all = "camelCase")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	pub id: i32,
	pub name: String,
	pub note: Option<String>,

	#[sea_orm(has_many)]
	pub children: HasMany<super::child::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
```

Compatibility behavior is scalar-only: `projection::ApiModel` contains `name` and `note`, not the
relationship field copied into SeaORM's `ModelEx`. Generic projection items are emitted once.

The generated inherent conveniences preserve current behavior:

```rust,ignore
let omitted = projection::ApiModel {
	name: "Athena".to_owned(),
	note: None,
}
.to_active_model();

assert!(omitted.id.is_not_set());
assert!(omitted.note.is_not_set());

let explicit_null = projection::ApiModel {
	name: "Athena".to_owned(),
	note: Some(None),
}
.to_active_model();

assert_eq!(explicit_null.note, sea_orm::ActiveValue::Set(None));
```

`to_model(missing)` completes into scalar `Model`; `to_active_model()` converts to scalar
`ActiveModel`. The projection also implements `projected_seaorm::SeaOrmProjection` and
`From<Projection> for ActiveModel`. Relationship-aware `ModelEx`/`ActiveModelEx` generation is not
provided.
