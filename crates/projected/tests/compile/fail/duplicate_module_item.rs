use projected::projected;

#[projected(module, projections(First))]
struct FirstModel {
	value: i32,
}

#[projected(module, projections(Second))]
struct SecondModel {
	value: i32,
}

fn main() {}
