use projected::projected;

#[projected(projections(Api(include(value, value))))]
struct Source {
	value: i32,
}

fn main() {}
