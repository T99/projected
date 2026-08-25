# Projected architecture

## Status and scope

This document is the architectural source of truth for the port of Athena's projection and
ordering support into the external `projected` workspace. The source baseline is Athena's
`backend/crates/core` and `backend/crates/proc-macros` implementation at the time of extraction;
the initial port described here is implemented in this workspace.

The extraction should preserve existing selection, completion, generic, attribute-propagation,
ordering-name, and SeaORM `ActiveModel` behavior unless this document or
[`public-api.md`](public-api.md) explicitly changes it. Athena consumers have not yet been
migrated, and `Filterable` is not implemented.

Deliberate public changes are:

- one public `#[projected(...)]` attribute replaces `Projectable`, `projectable_model`,
  `#[projectable]`, and `#[projection]`;
- `Projection` remains the public projection trait; `Projectable` disappears from public
  vocabulary;
- struct and field configuration both use `#[projected(...)]`;
- projection, ordering, future filtering, and shared query-field metadata live in one project;
- SeaORM is an optional integration, not a core assumption.

## Workspace and dependency boundaries

| Crate | Responsibility | Must not own |
| --- | --- | --- |
| `projected` | ORM-agnostic public runtime traits, types, metadata contracts, and re-export of `#[projected]` | `syn` models, SeaORM types, database-column mappings |
| `projected-derive` | The public attribute macro, parsing, normalization, orchestration, hidden expansion phases, and generic code generation | public runtime state, a SeaORM dependency in the generic model |
| `projected-seaorm` | Optional SeaORM runtime traits/adapters and ordinary generic integration code | public macro orchestration, generic projection rules |
| `projected-seaorm-codegen` | Internal token-generation library called by `projected-derive` for typed, entity-specific `ActiveModel` initialization | another independently invoked public proc macro or generic projection rules |

The implemented dependency direction is `projected -> projected-derive` for macro re-export,
`projected-seaorm -> projected + sea-orm`, and
`projected-derive -> projected-seaorm-codegen`. The codegen crate is an ordinary library used by
the orchestrator, not a second public attribute macro. Generated paths are resolved with
`proc-macro-crate`, including renamed direct dependencies. Feature-gated distribution remains an
open packaging question.

`projected-derive` remains the single orchestrator. Independent proc-macro crates cannot each
consume one `#[projected]` invocation or reliably share a resolved syntax tree.

## The five architectural layers

### 1. Public API

Downstream code sees:

- `#[projected(...)]` on a named-field struct and, when needed, its fields;
- generated owned projection structs and missing-field structs;
- `Projection`, `ProjectedModel`, `ProjectedField`, `Orderable`, `OrderBy`, and
  `OrderingDirection` from `projected`;
- optional SeaORM extension traits or methods from `projected-seaorm`.

There is no public `Projectable` trait or derive. Safe derive inheritance is performed by the
attribute macro, so a separate `projectable_model` phase is not exposed. See
[`public-api.md`](public-api.md) for intended syntax.

### 2. Compile-time internal representation

`projected-derive` retains Athena's parse-then-resolve organization, generalized around a
single model:

1. `SourceModel` / `SourceField` capture the named struct, generics, visibility, source order,
   attributes, and raw `#[projected]` configuration.
2. Projection declarations and field rules resolve into selections with explicit rule origins.
3. A shared resolved field record computes once:
   - Rust identifier, type, declaration index, and conditional attributes;
   - directional Serde-visible serialize and deserialize names;
   - propagation policy;
   - projection membership and optionality per projection;
   - ordering eligibility and, later, filtering eligibility;
   - backend-neutral identity used by generated metadata.
4. `ResolvedModel` validates conflicts before any tokens are emitted.
5. Generic projection and ordering emitters consume only the resolved representation.

This representation is proc-macro-internal and may contain `syn`/`proc_macro2` values. It is not
the runtime API. Athena's current `SourceModel`, `ProjectionDeclaration`, `ResolvedProjection`,
`ResolvedField`, `RuleOrigin`, `Selection`, and `AttributePolicy` are the starting point.

Serde names are resolved here once. Athena's `Orderable` implementation independently parsed
`serde(rename)` and `serde(rename_all)`; the port promotes that parser into shared field resolution.
Projection, ordering, future `Filterable`, and backend integrations must not each invent their own
name interpretation.

### 3. Generated runtime field representation

Every annotated model and generated projection exposes backend-neutral metadata through
`ProjectedModel`, `ProjectedField`, and `ProjectedFieldMapping`. The implemented contract provides:

- a generated field identity for each logical Rust field;
- deterministic source declaration order;
- Rust field name;
- resolved Serde serialize and deserialize names;
- query capabilities such as orderable (and later filterable), without backend types;
- a way for a generated projection field to retain the identity of its source-model field.

The current runtime shape is:

```rust,ignore
pub trait ProjectedModel {
	type Field: ProjectedField;
	fn fields() -> &'static [Self::Field];
}

pub trait ProjectedField: Copy + Eq + core::fmt::Debug + 'static {
	fn metadata(self) -> FieldMetadata;
}

pub trait ProjectedFieldMapping<Source: ProjectedModel>: ProjectedField {
	fn source_field(self) -> Source::Field;
}
```

`FieldMetadata` contains `rust_name`, directional `serialize_name` and `deserialize_name`, and
`orderable`. The implementation generates a `{Type}Field` enum and returns its values in source
declaration order. Projection field enums map to the corresponding source field enum. It does not
put a SeaORM `Column`, SQL identifier, Diesel expression, or other database mapping into the
generic field type; those mappings belong to backend integrations.

`Orderable` should refer to this representation rather than define an unrelated field universe.
Its existing runtime data types remain a useful baseline:

```rust,ignore
pub trait Orderable: ProjectedModel {
	type OrderingField: /* maps to Self::Field */;
}

pub struct OrderBy<T: Orderable> {
	pub field: T::OrderingField,
	pub direction: OrderingDirection,
}
```

The generated `{Type}OrderField` enum and its wire behavior may be retained initially, provided
each variant maps losslessly to the model's shared field identity. Future `Filterable` is only a
consumer used to validate this boundary; it is not part of the extraction implementation.

### 4. Backend integration boundary

Generic core describes logical fields and projection values. A backend integration maps those
logical fields to backend concepts and supplies backend-specific operations.

For SeaORM this boundary includes, if supportable through ordinary traits:

- mapping a shared field identity to an entity `Column` when the field is scalar;
- converting an ordering request to a SeaORM order expression;
- converting a projection to `ActiveModel` while preserving `Set` versus `NotSet` semantics;
- distinguishing scalar model fields from relationship fields.

The generic model must not treat a field as a database column merely because its Rust identifier
resembles one. Column renames, skipped fields, relationships, and backend-specific expressions
make that assumption invalid.

The port proved that current `ActiveModel` compatibility needs typed per-entity initializers:
required fields, projection-optional fields, and excluded fields produce different token shapes,
and each assignment names a generated `ActiveModel` field. That generation lives in
`projected-seaorm-codegen`. `projected-derive` constructs its backend-neutral input and invokes it,
so it remains the only public orchestrator. `projected-seaorm` owns the public `SeaOrmProjection`
runtime trait. Whether column and ordering mappings can use ordinary Rust traits is still open and
does not justify adding more generated mappings yet.

Other integrations (Diesel, SQLx metadata, raw SQL) should be able to define equivalent mapping
traits without changing `ProjectedField`.

### 5. Hidden multi-phase macro machinery

The final API must preserve the ability to generate against SeaORM's post-`#[sea_orm::model]`
representation while exposing only `#[projected]`.

The implemented expansion protocol is:

1. The public `#[projected]` attribute runs first, captures its complete configuration and the
   original derive list, and resolves the safe derives to inherit.
2. It rewrites field controls to `#[projected_internal(...)]`, adds hidden `__Projected` derive and
   helper metadata, and removes the public control attribute. Helper names are implementation
   details.
3. For a plain struct, the hidden derive performs normal resolution and emission.
4. For a SeaORM struct, `#[sea_orm::model]` runs after `#[projected]`, transforms the struct, and
   copies the hidden derive/helper metadata to the generated `Model` and `ModelEx`.
5. The hidden derive runs on those post-SeaORM structs. It emits generic metadata/projections once
   for canonical scalar `Model`, and suppresses duplicate generic emission for `ModelEx` unless a
   future explicit mode requests relationship-bearing projections.
6. The orchestrator emits or requests the optional SeaORM integration only after the generic model
   has resolved.

This replaces the current public `#[projectable_model]` + `#[derive(Projectable)]` pairing. Hidden
derives and helper attributes should carry normalized data rather than forcing a later phase to
reparse the original public syntax independently. UI tests must lock down phase ordering and
duplicate suppression.

## Existing behavior to preserve

### Projection behavior

Athena currently supports named-field structs; multiple declarations; `include`, `exclude`, and
`optional`; field rules targeted to one, many, or all projections; source-order preservation;
generic, lifetime, const-generic, and `where` preservation; and `cfg`/`cfg_attr` consistency.

An optional projection field adds an outer `Option`. Thus source `Option<T>` becomes
`Option<Option<T>>`: outer `None` means omitted, `Some(None)` means explicit null. Completion uses
the fallback only for outer `None`.

Each generated projection:

- implements `Projection<Base = Source, Missing = ...>`;
- implements `From<Source>` and wraps projection-optional values in `Some`;
- has `complete_with(...)` in source-field order;
- uses `()` and supports `into_base`/`From<Projection>` when lossless;
- otherwise gets a public `{Projection}MissingFields` type and `complete(missing_fields)`;
- gets a hidden `PhantomData` field when selected fields do not use every generic parameter;
- copies source visibility and optionally lives in a `projection` or custom module;
- propagates `doc`, `cfg`, and `cfg_attr` by default, and configured Serde/schema attributes.

Existing conflict diagnostics—unknown/duplicate fields or projections, include/exclude conflicts,
duplicate explicit rules, malformed module options, and unsupported tuple/unit structs—should move
with the implementation.

### Ordering behavior

Athena's `Orderable` generates `{Type}OrderField`, skips `#[orderable(skip)]` fields, and derives
debug/clone/copy/equality/hash plus Serde. Wire names honor field `serde(rename)`, container
`serde(rename_all)`, directional serialize/deserialize forms, and field rename precedence. The
new architecture should preserve those results while moving skip configuration under field-level
`#[projected(...)]` and resolving names in shared metadata.

`OrderingDirection` remains `Ascending`/`Descending`, serialized in camelCase with `asc`/`desc`
aliases. `OrderBy<T>` remains `{ field, direction }`.

### Current SeaORM expansion behavior

SeaORM 2.0 dense `#[sea_orm::model]` expansion produces two structs:

- scalar `Model`, with relationship fields removed and `#[sea_orm(model_ex)]` added;
- relationship-bearing `ModelEx`, retaining scalar and `BelongsTo`/`HasOne`/`HasMany` fields and
  replacing `DeriveEntityModel` with `DeriveModelEx`/`DeriveActiveModelEx`.

SeaORM copies third-party derives and non-SeaORM attributes to both structs. Athena deliberately
places `#[projectable_model]` before `#[sea_orm::model]`, while `Projectable` remains in the derive
list. As a result, `Projectable` sees the post-transform structs.

Athena detects scalar `Model` through `#[sea_orm(model_ex)]`. It detects the companion by the
combination of table/schema metadata, absence of that marker, and an identifier ending in `Ex`,
then suppresses the companion expansion. `#[projectable(emit)]` bypasses this name-based
suppression; `#[projectable(sea_orm)]` forces SeaORM emission when detection fails. The final API
does not owe these escape-hatch spellings compatibility.

Consequences of the current behavior:

- projections of dense entities contain scalar fields only; relationship fields are not projected;
- projection declarations are validated against scalar `Model`; the suppressed `ModelEx` is not
  resolved or validated;
- projections and their modules are emitted once despite the cloned derive;
- `From<Projection> for ActiveModel` sets included required fields to `Set(value)`, included
  projection-optional fields to `Set(value)` for `Some(value)` and `NotSet` for `None`, and leaves
  excluded fields `NotSet` through `Default`;
- for source `Option<T>`, projection `Some(None)` becomes `Set(None)`, while outer `None` remains
  `NotSet`;
- `to_active_model()` delegates to that conversion;
- `to_model(...)` is completion into scalar `Model`, not conversion from `ActiveModel`;
- relationship-aware `ActiveModelEx` is not targeted.

The port's unit/UI tests directly cover single-emission, the existence of `ModelEx`, scalar-only
metadata, and scalar `ActiveModel` semantics with `BelongsTo` and `HasMany`. Athena's compiled
database crate additionally exercises dense models with `BelongsTo` and `HasMany` fields
(`event`, `district`, `user`, `session`, and `tenant`), but it does not directly assert their
expanded projection field lists or `ModelEx` APIs. The extraction test plan must add those direct
assertions for every relationship form.

## Open Questions / Investigation Required

1. **SeaORM column and ordering mappings.** Determine whether ordinary traits can map
   `{Type}Field`/`{Type}OrderField` to `Column`, including renamed columns, raw identifiers,
   composite keys, skipped fields, and `cfg`. Do not add entity-specific token generation until an
   experiment demonstrates it is required.
2. **Runtime metadata evolution.** Validate whether returning `FieldMetadata` by value and a static
   slice of generated enum values remains sufficient for non-SeaORM integrations. Avoid adding
   backend data or field-type `'static` bounds.
3. **Ordering surface stability.** The implemented opt-in is `#[projected(orderable)]` for the
   source and `projection_derives(projected::Orderable)` for projections. Decide before a stable
   release whether `{Type}OrderField` is a guaranteed public generated name.
4. **Serde surface.** The resolver handles `rename` and `rename_all`, including directional
   forms. Define behavior for aliases, `skip`, `flatten`, and conflicting Serde attributes before
   claiming complete Serde equivalence. Serialization and deserialization names may differ and
   must not be collapsed.
5. **Relationship fields.** Scalar-only projection is the compatibility baseline. Determine whether
   projecting `ModelEx` relationships is ever desirable and, if so, require an explicit mode rather
   than silently changing existing dense-model output.
6. **Reliable SeaORM phase detection.** Replace or validate Athena's `*Ex` naming heuristic. Test
   `model_attrs`/`model_ex_attrs`, custom attributes, compact models, and models whose legitimate
   source name ends in `Ex`.
7. **Backend discovery and packaging.** Generated code locates renamed direct dependencies through
   `proc-macro-crate`. Decide feature wiring and improve the diagnostic when an annotated SeaORM
   model lacks `projected-seaorm` or `sea-orm` as a direct dependency.
8. **Safe derive inheritance.** The current allowlist is `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`,
   `PartialOrd`, `Ord`, `Hash`, `Default`) and path de-duplication behavior. Non-safe derives remain
   explicit projection configuration.
9. **Additional relationship coverage.** Add direct fixtures for optional `BelongsTo`, `HasOne`,
   via relations, compact models, and legitimate source names ending in `Ex` before changing the
   scalar-only compatibility baseline.
