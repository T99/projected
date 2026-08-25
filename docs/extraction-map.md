# Athena-to-`projected` extraction map

Paths in the first column are relative to Athena's `backend/`; destinations are relative to this
workspace. This records the initial port, not a request to remove Athena's implementation. Athena
remains the behavior oracle until its consumers are migrated in a later task.

## Workspace and runtime

| Athena source | Ported destination | Notes |
| --- | --- | --- |
| workspace dependency entries in `Cargo.toml` | root `Cargo.toml` and member manifests | Generic crates have no SeaORM dependency; SeaORM is confined to the integration crate and its tests. |
| `crates/core/src/traits/projection.rs` | `crates/projected/src/projection.rs` | `Projection`, completion behavior, and associated `Base`/`Missing` types are preserved. `Projectable` is removed. |
| `crates/core/src/traits/orderable.rs` | `crates/projected/src/order.rs` | `Orderable`, `OrderingDirection`, and `OrderBy` retain their wire behavior; `Orderable` now maps to shared fields. |
| no direct equivalent | `crates/projected/src/field.rs` | Adds backend-neutral `FieldMetadata`, `ProjectedField`, `ProjectedModel`, and `ProjectedFieldMapping`. |
| macro facade/re-exports | `crates/projected/src/lib.rs` | Re-exports one public `#[projected]` attribute and the runtime API. Hidden `__Projected` is an implementation detail. |

## Proc-macro orchestration and generic generation

| Athena source | Ported destination | Notes |
| --- | --- | --- |
| `crates/proc-macros/src/lib.rs`, `projectable/mod.rs` | `crates/projected-derive/src/lib.rs`, `src/projected/mod.rs` | Replaced public `projectable_model`, `Projectable`, and `Orderable` macro entry points with public `#[projected]` plus a hidden derive phase. |
| `projectable/models/source_model.rs` | `projected/models/source_model.rs` | Preserves named structs, generics, projection declaration/rule parsing, validation, and SeaORM post-transform detection. |
| `projection_declaration.rs`, `raw_projection_modifier.rs` | same filenames under `projected/models/` | Preserves `include`, `exclude`, and `optional` semantics under `projections(...)`. |
| `raw_field_rule.rs`, `field_action.rs` | same filenames under `projected/models/` | Struct and field controls now share `#[projected(...)]`; ordering skip is `order(skip)`. |
| `resolved_field.rs`, `rule_origin.rs` | same filenames under `projected/models/` | Preserves explicit/implicit rule origins and conflict diagnostics. |
| `resolved_model.rs`, `resolved_projection.rs` | same filenames under `projected/models/` | Preserves module placement, selection, completion, conversions, generic markers, and dispatches generic/backend emitters. |
| `attribute_policy.rs`, `attributes.rs` | `projected/models/attribute_policy.rs`, `projected/attributes.rs` | Preserves safe derive inheritance and controlled `doc`, `cfg`, `cfg_attr`, Serde, and schema propagation. |
| `projectable/names.rs` | `projected/names.rs` | Generates projection support names, field enums, ordering enums, and collision-safe fallback/marker identifiers. |
| `orderable/serde_name.rs` | `projected/serde_name.rs` | Serde `rename`/`rename_all`, including directional forms, is resolved once into each `SourceField`. |
| `orderable/expand.rs` | `projected/codegen/order.rs` | Emits `{Type}OrderField` from shared resolved metadata and maps each value to `{Type}Field`. |
| no direct equivalent | `projected/codegen/metadata.rs` | Emits `{Type}Field`, its metadata, deterministic model field lists, and projection-to-source mappings. |

The main internal types remain close to Athena: `SourceModel`, `ProjectionDeclaration`,
`ResolvedModel`, `ResolvedProjection`, `ResolvedField`, `RuleOrigin`, `Selection`, and
`AttributePolicy`. The important change is that `SourceField` owns resolved directional Serde names
and query eligibility so ordering and later `Filterable` do not reparse attributes.

## Shared utilities

| Athena source | Ported destination | Notes |
| --- | --- | --- |
| `util/generic_usage.rs` | `crates/projected-derive/src/util/generic_usage.rs` | Preserves lifetime/type/const usage detection for hidden `PhantomData`. |
| `util/parse_ident_list.rs` | `crates/projected-derive/src/util/parse_ident_list.rs` | Reused by projection declarations and field targets. |
| `util/real_mod_path.rs` | `crates/projected-derive/src/util/crate_path.rs` | Uses `proc-macro-crate` for `projected`, `projected-seaorm`, and `sea-orm`, including renamed dependencies. |
| `util/struct_with_named_fields.rs`, `util/field.rs` | folded into `SourceModel` | Ordering no longer maintains a second independent struct parser. |

## SeaORM integration

| Athena source/behavior | Ported destination | Notes |
| --- | --- | --- |
| `SourceModel::detect_sea_orm`, `SeaOrmInfo` | `projected/models/source_model.rs` | Preserves scalar `Model` detection, `ModelEx` suppression, and `emit`/`sea_orm` internal escape behavior. The naming heuristic remains an open question. |
| `ResolvedProjection::emit_sea_orm` runtime contract | `crates/projected-seaorm/src/lib.rs` | `SeaOrmProjection` exposes typed `ActiveModel` conversion. |
| entity-specific `ActiveModel` token generation | `crates/projected-seaorm-codegen/src/lib.rs` | Kept separate because required, optional, excluded, and nullable fields require typed per-entity initializers. It is a library invoked only by `projected-derive`, not a public macro. |
| `From<Projection> for ActiveModel`, `to_active_model`, `to_model` | orchestrated between `resolved_projection.rs` and `projected-seaorm-codegen` | Preserves `Set`/`NotSet`, including outer `None` versus `Some(None)`, and targets scalar `ActiveModel` only. |
| dense SeaORM `Model`/`ModelEx` behavior | `crates/projected-seaorm/src/lib.rs` tests and `tests/ui/pass/sea_orm.rs` | Verifies relationship fields stay in `ModelEx`, scalar metadata/projections emit once, and `BelongsTo`/`HasMany` models compile. |

No generic `ProjectedField` contains a SeaORM `Column` or database name. Column and ordering
mappings remain future integration work; ordinary traits should be tested before extending
`projected-seaorm-codegen`.

## Tests and fixtures

| Athena coverage | Ported destination                                                     | Coverage |
| --- |------------------------------------------------------------------------| --- |
| projection unit tests in `crates/core/src/traits/projection.rs` | `crates/projected/tests/projection.rs`                                 | Selection, nested optional values, completion/fallback order, lossless conversion, field rules, generics/lifetimes/const generics/where clauses, `cfg`, propagation, modules, and metadata mapping. |
| ordering unit tests in `crates/core/src/traits/orderable.rs` | `crates/projected/tests/order.rs`                                      | Default names, container/field/directional Serde renames, field skip, projection ordering, and shared-field identity. |
| `crates/core/tests/projectable_ui.rs` | `crates/projected/tests/compile.rs`                                    | Trybuild pass/fail harness under the public `#[projected]` syntax. |
| `tests/ui/projectable/pass/plain.rs`, `modules.rs`, safe-derive fixtures | `crates/projected/tests/compile/pass/`                                 | Plain structs, modules, safe inheritance, and ordering. |
| all applicable `tests/ui/projectable/fail/*.rs` | `crates/projected/tests/compile/fail/`                                 | Conflicts, duplicates, malformed options/modules, unknown fields/projections/options, and tuple/unit rejection with regenerated snapshots. |
| `tests/ui/projectable/pass/sea_orm.rs` and SeaORM unit behavior | `crates/projected-seaorm/tests/`, `crates/projected-seaorm/src/lib.rs` | Public attribute ordering, dense expansion, duplicate suppression, completion, and `ActiveModel` semantics. |

Still useful before a stable release: add direct SeaORM fixtures for optional `BelongsTo`,
`HasOne`, via relations, compact models, column renames, composite keys, raw identifiers, and
dependency aliases. Add a backend-neutral mock mapping test before implementing `Filterable`.

## Athena consumer migration (later)

The Athena implementation and consumers are intentionally unchanged by this port. A later
migration can update entity annotations/imports and ordering consumers after choosing dependency
publication/path wiring. Do not remove the manual ordering-to-column match in
`crates/api/src/routes/v1/canonical/teams.rs` until `projected-seaorm` has a tested equivalent.
