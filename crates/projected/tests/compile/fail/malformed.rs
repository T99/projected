use projected::projected;

#[projected(projections(Api(include(value) optional(value))))]
struct Source {
	value: i32,
}

fn main() {}
