use projected::projected;

#[projected(projections(Api(include(value))))]
struct Source {
	#[projected(exclude(Api))]
	value: i32,
}

fn main() {}
