use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Reach new heights.
struct GoUp {
    /// whether or not to jump
    #[argh(switch, short = 'j')]
    jump: bool,

    /// how high to go
    #[argh(option, env = "GO_UP_HEIGHT")]
    height: usize,

    /// an optional nickname for the pilot
    #[argh(option, env = "GO_UP_PILOT", from_str_fn(parse_box_str))]
    pilot_nickname: Box<str>,
}

fn parse_box_str(s: &str) -> Result<Box<str>, String> {
    Ok(s.into())
}

fn main() {
    let up: GoUp = argh::from_env();
    println!("Options: {:?}", up);
}
