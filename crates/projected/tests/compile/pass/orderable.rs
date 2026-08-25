use projected::{Orderable, projected};

#[projected(orderable)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Model {
	field_name: String,
	#[projected(order(skip))]
	hidden: String,
}

fn assert_orderable<T: Orderable>() {}

fn main() {
	assert_orderable::<Model>();
	let _ = ModelOrderField::FieldName;
}
