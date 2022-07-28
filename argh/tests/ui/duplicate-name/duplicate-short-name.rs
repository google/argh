/// Command
#[derive(argh::FromArgs)]
struct Cmd {
    /// foo1
    #[argh(option, short = 'f')]
    foo1: u32,

    /// foo2
    #[argh(option, short = 'f')]
    foo2: u32,

    /// bar1
    #[argh(option, short = 'b')]
    bar1: u32,

    /// bar2
    #[argh(option, short = 'b')]
    bar2: u32,
}

fn main() {}
