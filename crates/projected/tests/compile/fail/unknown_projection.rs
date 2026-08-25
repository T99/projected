use projected::projected;

#[projected(projections(Api))]
struct Source {
	#[projected(exclude(Undeclared))]
	value: i32,
}

fn main() {}
