/// Command
#[derive(argh::FromArgs)]
struct Cmd {
    /// foo1
    #[argh(option, long = "foo")]
    foo1: u32,

    /// foo2
    #[argh(option, long = "foo")]
    foo2: u32,

    /// bar1
    #[argh(option, long = "bar")]
    bar1: u32,

    /// bar2
    #[argh(option, long = "bar")]
    bar2: u32,
}

fn main() {}
