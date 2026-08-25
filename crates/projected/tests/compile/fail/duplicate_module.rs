use projected::projected;

#[projected(module, module = views, projections(Public))]
struct Model {
	value: i32,
}

fn main() {}
