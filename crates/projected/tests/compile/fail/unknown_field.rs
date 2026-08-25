use projected::projected;

#[projected(projections(Api(include(missing))))]
struct Source {
	present: i32,
}

fn main() {}
