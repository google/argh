/// A `dynamic` attribute is only valid on a variant with a single unnamed
/// field; applying it to a struct-style variant is an error.
#[derive(argh::FromArgs)]
#[argh(subcommand)]
enum Cmd {
    /// a thing
    #[argh(dynamic)]
    Thing {
        /// a value
        #[argh(positional)]
        value: String,
    },
}

fn main() {}
