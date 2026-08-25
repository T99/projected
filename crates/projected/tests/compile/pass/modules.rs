use projected::projected;

pub mod public_model {
	use super::*;

	pub type Name = String;

	#[projected(
		module,
		projections(Public(exclude(secret)), Summary(include(id, name)))
	)]
	pub struct Model {
		pub id: i32,
		pub name: Name,
		pub secret: String,
	}
}

mod crate_model {
	use super::*;

	#[projected(module = views, projections(Public))]
	pub(crate) struct Model {
		pub value: i32,
	}
}

fn main() {
	let public = public_model::projection::Public::from(public_model::Model {
		id: 1,
		name: "Athena".to_owned(),
		secret: "secret".to_owned(),
	});
	let _ = public.complete_with("secret".to_owned());
	let _ = public_model::projection::Summary {
		id: 1,
		name: "Athena".to_owned(),
	};

	let projection = crate_model::views::Public::from(crate_model::Model { value: 2 });
	let _ = projection.into_base();
}
