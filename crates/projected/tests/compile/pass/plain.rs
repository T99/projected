use projected::{Projection, projected};

#[projected(
	projections(
		Included(include(name), optional(value)),
		Excluded(exclude(id)),
		Bare,
		EverythingOptional(optional)
	)
)]
#[derive(Debug, PartialEq)]
struct Plain<'a, T, const N: usize>
where
	T: PartialEq,
{
	id: u32,
	name: &'a str,
	value: T,
	bytes: [u8; N],
}

#[projected(projections(One(include(first)), Two))]
struct Fields {
	first: i32,
	#[projected(include(One), exclude(Two))]
	second: String,
	#[projected(optional)]
	third: Option<bool>,
}

fn assert_projection<P: Projection<Base = Plain<'static, i32, 2>>>() {}

fn main() {
	assert_projection::<Bare<'static, i32, 2>>();
	let base = Plain {
		id: 1,
		name: "plain",
		value: 2,
		bytes: [3, 4],
	};
	let projection = Included::from(base);
	let _: Option<i32> = projection.value;
	let _ = projection.complete_with(1, 0, [3, 4]);

	let fields = One {
		first: 1,
		second: String::new(),
		third: Some(Some(true)),
	};
	let _ = fields.complete_with(None);
}
