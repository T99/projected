#![deny(missing_docs)]

//! Verifies that public macro output remains usable under strict documentation lints.

use projected::projected;

/// A fully documented source model used to audit generated public items.
#[projected(
	module,
	orderable,
	projections(Public(exclude(secret), optional(name))),
	projection_derives(projected::Orderable)
)]
pub struct Model {
	/// Stable model identifier.
	pub id: i32,
	/// Human-readable name.
	pub name: String,
	/// Value omitted from the public projection and ordering API.
	#[projected(order(skip))]
	pub secret: String,
}

fn main() {
	let _ = ModelField::Id;
	let _ = ModelOrderField::Id;
	let _ = projection::PublicField::Name;
	let _ = projection::PublicOrderField::Name;
}
