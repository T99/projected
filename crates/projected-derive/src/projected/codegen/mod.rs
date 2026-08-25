/// Shared logical-field metadata generation.
mod metadata;
/// Ordering enum and mapping generation.
mod order;

pub use metadata::{MetadataField, MetadataInput};
pub use order::{OrderField, OrderInput};
