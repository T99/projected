use projected::projected;

#[projected(projections(Api, Api(exclude(value))))]
struct Source {
	value: i32,
}

fn main() {}
