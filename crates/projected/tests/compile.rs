#[test]
fn projected_compile_fixtures() {
	let tests = trybuild::TestCases::new();
	tests.pass("tests/compile/pass/*.rs");
	tests.compile_fail("tests/compile/fail/*.rs");
}
