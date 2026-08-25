//! Backend-neutral model projections and shared query-field metadata.

#![warn(missing_docs)]

extern crate self as projected;

mod field;
mod order;
mod projection;

pub use field::{FieldMetadata, ProjectedField, ProjectedFieldMapping, ProjectedModel};
pub use order::{OrderBy, Orderable, OrderingDirection};
pub use projected_derive::projected;
pub use projection::Projection;

#[doc(hidden)]
pub use projected_derive::__Projected;

#[projected(
	projections(
		PartialOptional(include(name), optional(age)),
		OptionalFields(optional(name, age))
	)
)]
struct MyStruct {
	pub name: String,
	pub age: u32,
	pub email: String,
}
