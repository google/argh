/// Command
#[derive(argh::FromArgs)]
struct Cmd {
    #[argh(positional)]
    /// positional
    positional: Vec<String>,

    #[argh(positional, greedy)]
    /// remainder
    remainder: Vec<String>,
}

fn main() {}
