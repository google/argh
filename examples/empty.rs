use argh::FromArgs;

#[derive(FromArgs)]
/// Empty example for testing.
struct Empty {}

fn main() {
	let Empty {} = argh::from_env();
}
