use projected::*;

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

fn asd() {
	let a = MyStruct {
		name: "Alice".to_string(),
		age: 30,
		email: "alice@example.com".to_string(),
	};
}