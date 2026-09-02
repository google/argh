/// Tuple variants must contain exactly one field; a multi-field tuple variant
/// is an error.
#[derive(argh::FromArgs)]
#[argh(subcommand)]
enum Cmd {
    Pair(u32, u32),
}

fn main() {}
