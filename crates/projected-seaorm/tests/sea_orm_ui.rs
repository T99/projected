#[test]
fn sea_orm_compile_fixtures() {
	let tests = trybuild::TestCases::new();
	tests.pass("tests/compile/pass/*.rs");
}
