/// Command
#[derive(argh::FromArgs)]
struct Cmd {
    #[argh(switch)]
    /// non-ascii
    привет: bool,
    #[argh(switch)]
    /// uppercase
    XMLHTTPRequest: bool,
    #[argh(switch, long = "not really")]
    /// bad attr
    ok: bool,
}

fn main() {}
