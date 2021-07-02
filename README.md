# Argh
**Argh is an opinionated Derive-based argument parser optimized for code size**

[![crates.io](https://img.shields.io/crates/v/cargho.svg)](https://crates.io/crates/cargho)
[![license](https://img.shields.io/badge/license-BSD3.0-blue.svg)](https://github.com/google/cargho/LICENSE)
[![docs.rs](https://docs.rs/cargho/badge.svg)](https://docs.rs/crate/cargho/)
![Argh](https://github.com/google/cargho/workflows/Argh/badge.svg)

Derive-based argument parsing optimized for code size and conformance
to the Fuchsia commandline tools specification

The public API of this library consists primarily of the `FromArgs`
derive and the `from_env` function, which can be used to produce
a top-level `FromArgs` type from the current program's commandline
arguments.

## Basic Example

```rust,no_run
use cargho::FromArgs;

#[derive(FromArgs)]
/// Reach new heights.
struct GoUp {
    /// whether or not to jump
    #[cargho(switch, short = 'j')]
    jump: bool,

    /// how high to go
    #[cargho(option)]
    height: usize,

    /// an optional nickname for the pilot
    #[cargho(option)]
    pilot_nickname: Option<String>,
}

fn main() {
    let up: GoUp = cargho::from_env();
}
```

`./some_bin --help` will then output the following:

```
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
`#[cargho(default = "<your_code_here>")]` attribute, and in this case an
option is treated as optional.

```rust
use cargho::FromArgs;

fn default_height() -> usize {
    5
}

#[derive(FromArgs)]
/// Reach new heights.
struct GoUp {
    /// an optional nickname for the pilot
    #[cargho(option)]
    pilot_nickname: Option<String>,

    /// an optional height
    #[cargho(option, default = "default_height()")]
    height: usize,

    /// an optional direction which is "up" by default
    #[cargho(option, default = "String::from(\"only up\")")]
    direction: String,
}

fn main() {
    let up: GoUp = cargho::from_env();
}
```

Custom option types can be deserialized so long as they implement the
`FromArgValue` trait (automatically implemented for all `FromStr` types).
If more customized parsing is required, you can supply a custom
`fn(&str) -> Result<T, String>` using the `from_str_fn` attribute:

```rust
use cargho::FromArgs;

#[derive(FromArgs)]
/// Goofy thing.
struct FiveStruct {
    /// always five
    #[cargho(option, from_str_fn(always_five))]
    five: usize,
}

fn always_five(_value: &str) -> Result<usize, String> {
    Ok(5)
}
```

Positional arguments can be declared using `#[cargho(positional)]`.
These arguments will be parsed in order of their declaration in
the structure:

```rust
use cargho::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// A command with positional arguments.
struct WithPositional {
    #[cargho(positional)]
    first: String,
}
```

The last positional argument may include a default, or be wrapped in
`Option` or `Vec` to indicate an optional or repeating positional argument.

Subcommands are also supported. To use a subcommand, declare a separate
`FromArgs` type for each subcommand as well as an enum that cases
over each command:

```rust
use cargho::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Top-level command.
struct TopLevel {
    #[cargho(subcommand)]
    nested: MySubCommandEnum,
}

#[derive(FromArgs, PartialEq, Debug)]
#[cargho(subcommand)]
enum MySubCommandEnum {
    One(SubCommandOne),
    Two(SubCommandTwo),
}

#[derive(FromArgs, PartialEq, Debug)]
/// First subcommand.
#[cargho(subcommand, name = "one")]
struct SubCommandOne {
    #[cargho(option)]
    /// how many x
    x: usize,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Second subcommand.
#[cargho(subcommand, name = "two")]
struct SubCommandTwo {
    #[cargho(switch)]
    /// whether to fooey
    fooey: bool,
}
```

NOTE: This is not an officially supported Google product.
