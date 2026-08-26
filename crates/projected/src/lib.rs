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
