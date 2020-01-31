# Argh
**Argh is an opinionated Derive-based argument parser optimized for code size**

![crates.io](https://img.shields.io/crates/v/argh.svg)

Derive-based argument parsing optimized for code size and conformance
to the Fuchsia commandline tools specification

The public API of this library consists primarily of the `FromArgs`
derive and the `from_env` function, which can be used to produce
a top-level `FromArgs` type from the current program's commandline
arguments.

## Basic Example

```rust,no_run
use argh::FromArgs;

#[derive(FromArgs)]
/// Reach new heights.
struct GoUp {
    /// whether or not to jump
    #[argh(switch, short = 'j')]
    jump: bool,

    /// how high to go
    #[argh(option)]
    height: usize,

    /// an optional nickname for the pilot
    #[argh(option)]
    pilot_nickname: Option<String>,
}

fn main() {
    let up: GoUp = argh::from_env();
}
```

`./some_bin --help` will then output the following:

```bash
Usage: cmdname [-j] --height <height> [--pilot-nickname <pilot-nickname>]

Reach new heights.

Options:
  -j, --jump        whether or not to jump
  --height          how high to go
  --pilot-nickname  an optional nickname for the pilot
  --help            display usage information
```

The resulting program can then be used in any of these ways:
- `./some_bin --height 5`
- `./some_bin -j --height 5`
- `./some_bin --jump --height 5 --pilot-nickname Wes`

Switches, like `jump`, are optional and will be set to true if provided.

Options, like `height` and `pilot_nickname`, can be either required,
optional, or repeating, depending on whether they are contained in an
`Option` or a `Vec`. Default values can be provided using the
`#[argh(default = "<your_code_here>")]` attribute.

Custom option types can be deserialized so long as they implement the
`FromArgValue` trait (automatically implemented for all `FromStr` types).
If more customized parsing is required, you can supply a custom
`fn(&str) -> Result<T, String>` using the `from_str_fn` attribute:

```
# use argh::FromArgs;

#[derive(FromArgs)]
/// Goofy thing.
struct FiveStruct {
    /// always five
    #[argh(option, from_str_fn(always_five))]
    five: usize,
}

fn always_five(_value: &str) -> Result<usize, String> {
    Ok(5)
}
```

Positional arguments can be declared using `#[argh(positional)]`.
These arguments will be parsed in order of their declaration in
the structure:

```rust
use argh::FromArgs;
#[derive(FromArgs, PartialEq, Debug)]
/// A command with positional arguments.
struct WithPositional {
    #[argh(positional)]
    first: String,
}
```

The last positional argument may include a default, or be wrapped in
`Option` or `Vec` to indicate an optional or repeating positional arugment.

Subcommands are also supported. To use a subcommand, declare a separate
`FromArgs` type for each subcommand as well as an enum that cases
over each command:

```rust
# use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Top-level command.
struct TopLevel {
    #[argh(subcommand)]
    nested: MySubCommandEnum,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum MySubCommandEnum {
    One(SubCommandOne),
    Two(SubCommandTwo),
}

#[derive(FromArgs, PartialEq, Debug)]
/// First subcommand.
#[argh(subcommand, name = "one")]
struct SubCommandOne {
    #[argh(option)]
    /// how many x
    x: usize,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Second subcommand.
#[argh(subcommand, name = "two")]
struct SubCommandTwo {
    #[argh(switch)]
    /// whether to fooey
    fooey: bool,
}


```rust
# use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Top-level command.
struct TopLevel {
    #[argh(subcommand)]
    nested: MySubCommandEnum,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum MySubCommandEnum {
    One(SubCommandOne),
    Two(SubCommandTwo),
}

#[derive(FromArgs, PartialEq, Debug)]
/// First subcommand.
#[argh(subcommand, name = "one")]
struct SubCommandOne {
    #[argh(option)]
    /// how many x
    x: usize,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Second subcommand.
#[argh(subcommand, name = "two")]
struct SubCommandTwo {
    #[argh(switch)]
    /// whether to fooey
    fooey: bool,
}
```

NOTE: This is not an officially supported Google product.
