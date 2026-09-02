/// A unit struct used as a top-level command is not allowed; unit structs are
/// only supported as subcommands (requiring `#[argh(subcommand)]`).
#[derive(argh::FromArgs)]
struct Cmd;

fn main() {}
