# Argh
**Argh is an opinionated Derive-based argument parser optimized for code size**

[![crates.io](https://img.shields.io/crates/v/argh.svg)](https://crates.io/crates/argh)
[![license](https://img.shields.io/badge/license-BSD3.0-blue.svg)](https://github.com/google/argh/LICENSE)
[![docs.rs](https://docs.rs/argh/badge.svg)](https://docs.rs/crate/argh/)
![Argh](https://github.com/google/argh/workflows/Argh/badge.svg)

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
`#[argh(default = "<your_code_here>")]` attribute, and in this case an
option is treated as optional.

```rust
use argh::FromArgs;

fn default_height() -> usize {
    5
}

#[derive(FromArgs)]
/// Reach new heights.
struct GoUp {
    /// an optional nickname for the pilot
    #[argh(option)]
    pilot_nickname: Option<String>,

    /// an optional height
    #[argh(option, default = "default_height()")]
    height: usize,

    /// an optional direction which is "up" by default
    #[argh(option, default = "String::from(\"only up\")")]
    direction: String,
}

fn main() {
    let up: GoUp = argh::from_env();
}
```

Custom option types can be deserialized so long as they implement the
`FromArgValue` trait (automatically implemented for all `FromStr` types).
If more customized parsing is required, you can supply a custom
`fn(&str) -> Result<T, String>` using the `from_str_fn` attribute:

```rust
use argh::FromArgs;

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
`Option` or `Vec` to indicate an optional or repeating positional argument.

Subcommands are also supported. To use a subcommand, declare a separate
`FromArgs` type for each subcommand as well as an enum that cases
over each command:

```rust
use argh::FromArgs;

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

## Attribute Summary
### Type attributes for `argh`

The attributes used to configure the argh information for a type are defined in
[parse_attrs::TypeAttrs](argh_derive/src/parse_attrs.rs).

* `subcommand` - a subcommand type. This attribute must appear on both enumeration and each struct that
    is a variant for the enumerated subcommand.
* `error_code(code, description)` - an error code for the command. This attribute can appear zero
    or more times.
* `examples=` - Formatted text containing examples of how to use this command. This
   is an optional attribute.
* `name=` - (required for subcommand variant) the name of the subcommand.
* `notes=` - Formatted text containing usage notes for this command. This
   is an optional attribute.
    pub error_codes: Vec<(syn::LitInt, syn::LitStr)>,

### Field attributes for `argh`

The attributes used to configure the argh information for a field are
defined in [parse_attrs.rs](argh_derive/src/parse_attrs.rs).

* Field kind. This is the first attribute. Valid kinds are:
   * `switch` - a boolean flag, its presence on the command sets the field to `true`.
   * `option` - a value. This can be a simple type like String, or usize, and enumeration.
       This can be a scalar or Vec<>  for repeated values.
   * `subcommand` - a subcommand. The type of this field is an enumeration with a value for each
       subcommand. This attribute must appear on both the "top level" field and each struct that
       is a variant for the enumerated subcommand.
   * `positional` - a positional argument. This can be scalar or Vec<>. Only the last positional
       argument can be Option<>, Vec<>, or defaulted.
* `arg_name=` - the name to use for a positional argument in the help or the value of a `option`.
    If not given, the default is the name of the field.
* `default=` - the default value for the `option` or `positional` fields.
* `description=` - the description of the flag or argument. The default value is the doc comment
    for the field.
* `from_str_fn` is the name of a custom deserialization function for this field with the signature:
    `fn(&str) -> Result<T, String>`.
* `long=` - the long format of the option or switch name. If `long` is not present, the
    flag name defaults to the field name.
* `short=` - the single character for this flag. If `short` is not present, there is no
    short equivalent flag.


NOTE: This is not an officially supported Google product.
