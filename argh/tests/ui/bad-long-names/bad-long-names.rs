/// Command
#[derive(argh::FromArgs)]
struct Cmd {
    #[argh(switch)]
    /// non-ascii
    привет: bool,
    #[argh(switch)]
    /// invalid character
    XMLHTTPRequest: bool,
    #[argh(switch, long = "invalid_character")]
    /// bad attr
    ok: bool,
}

fn main() {}
