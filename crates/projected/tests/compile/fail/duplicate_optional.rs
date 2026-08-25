use projected::projected;

#[projected(projections(Api(optional(value), optional(value))))]
struct Source {
	value: i32,
}

fn main() {}
