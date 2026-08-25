use projected::projected;

#[projected(projections(Api(include(first), exclude(second))))]
struct Source {
	first: i32,
	second: i32,
}

fn main() {}
