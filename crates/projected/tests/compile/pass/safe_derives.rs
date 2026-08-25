use projected::projected;
use std::fmt::Debug;

#[projected(
	projections(ApiModel),
	projection_derives(Debug, Default)
)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct Model {
	value: i32,
}

fn assert_derives<T: Clone + Debug + PartialEq + Eq + Default>() {}

fn main() {
	assert_derives::<ApiModel>();
	let projection = ApiModel::from(Model { value: 7 });
	let duplicate = projection.clone();
	assert_eq!(projection, duplicate);
}
