// Copyright (c) 2020 Google LLC All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//! Derive-based argument parsing optimized for code size and conformance
//! to the Fuchsia commandline tools specification
//!
//! The public API of this library consists primarily of the `FromArgs`
//! derive and the `from_env` function, which can be used to produce
//! a top-level `FromArgs` type from the current program's commandline
//! arguments.
//!
//! ## Basic Example
//!
//! ```rust,no_run
//! use argh::FromArgs;
//!
//! #[derive(FromArgs)]
//! /// Reach new heights.
//! struct GoUp {
//!     /// whether or not to jump
//!     #[argh(switch, short = 'j')]
//!     jump: bool,
//!
//!     /// how high to go
//!     #[argh(option)]
//!     height: usize,
//!
//!     /// an optional nickname for the pilot
//!     #[argh(option)]
//!     pilot_nickname: Option<String>,
//! }
//!
//! fn main() {
//!     let up: GoUp = argh::from_env();
//! }
//! ```
//!
//! `./some_bin --help` will then output the following:
//!
//! ```bash
//! Usage: cmdname [-j] --height <height> [--pilot-nickname <pilot-nickname>]
//!
//! Reach new heights.
//!
//! Options:
//!   -j, --jump        whether or not to jump
//!   --height          how high to go
//!   --pilot-nickname  an optional nickname for the pilot
//!   --help            display usage information
//! ```
//!
//! The resulting program can then be used in any of these ways:
//! - `./some_bin --height 5`
//! - `./some_bin -j --height 5`
//! - `./some_bin --jump --height 5 --pilot-nickname Wes`
//!
//! Switches, like `jump`, are optional and will be set to true if provided.
//!
//! Options, like `height` and `pilot_nickname`, can be either required,
//! optional, or repeating, depending on whether they are contained in an
//! `Option` or a `Vec`. Default values can be provided using the
//! `#[argh(default = "<your_code_here>")]` attribute.
//!
//! Custom option types can be deserialized so long as they implement the
//! `FromArgValue` trait (automatically implemented for all `FromStr` types).
//! If more customized parsing is required, you can supply a custom
//! `fn(&str) -> Result<T, String>` using the `from_str_fn` attribute:
//!
//! ```
//! # use argh::FromArgs;
//!
//! #[derive(FromArgs)]
//! /// Goofy thing.
//! struct FiveStruct {
//!     /// always five
//!     #[argh(option, from_str_fn(always_five))]
//!     five: usize,
//! }
//!
//! fn always_five(_value: &str) -> Result<usize, String> {
//!     Ok(5)
//! }
//! ```
//!
//! Positional arguments can be declared using `#[argh(positional)]`.
//! These arguments will be parsed in order of their declaration in
//! the structure:
//!
//! ```rust
//! use argh::FromArgs;
//! #[derive(FromArgs, PartialEq, Debug)]
//! /// A command with positional arguments.
//! struct WithPositional {
//!     #[argh(positional)]
//!     first: String,
//! }
//! ```
//!
//! The last positional argument may include a default, or be wrapped in
//! `Option` or `Vec` to indicate an optional or repeating positional arugment.
//!
//! Subcommands are also supported. To use a subcommand, declare a separate
//! `FromArgs` type for each subcommand as well as an enum that cases
//! over each command:
//!
//! ```rust
//! # use argh::FromArgs;
//!
//! #[derive(FromArgs, PartialEq, Debug)]
//! /// Top-level command.
//! struct TopLevel {
//!     #[argh(subcommand)]
//!     nested: MySubCommandEnum,
//! }
//!
//! #[derive(FromArgs, PartialEq, Debug)]
//! #[argh(subcommand)]
//! enum MySubCommandEnum {
//!     One(SubCommandOne),
//!     Two(SubCommandTwo),
//! }
//!
//! #[derive(FromArgs, PartialEq, Debug)]
//! /// First subcommand.
//! #[argh(subcommand, name = "one")]
//! struct SubCommandOne {
//!     #[argh(option)]
//!     /// how many x
//!     x: usize,
//! }
//!
//! #[derive(FromArgs, PartialEq, Debug)]
//! /// Second subcommand.
//! #[argh(subcommand, name = "two")]
//! struct SubCommandTwo {
//!     #[argh(switch)]
//!     /// whether to fooey
//!     fooey: bool,
//! }
//! ```

#![deny(missing_docs)]

use std::str::FromStr;

pub use argh_derive::FromArgs;

/// Information about a particular command used for output.
pub type CommandInfo = argh_shared::CommandInfo<'static>;

/// Types which can be constructed from a set of commandline arguments.
pub trait FromArgs: Sized {
    /// Construct the type from an input set of arguments.
    ///
    /// The first argument `command_name` is the identifier for the current
    /// command, treating each segment as space-separated. This is to be
    /// used in the output of `--help`, `--version`, and similar flags.
    fn from_args(command_name: &[&str], args: &[&str]) -> Result<Self, EarlyExit>;
}

/// A top-level `FromArgs` implementation that is not a subcommand.
pub trait TopLevelCommand: FromArgs {}

/// A `FromArgs` implementation that can parse into one or more subcommands.
pub trait SubCommands: FromArgs {
    /// Info for the commands.
    const COMMANDS: &'static [&'static CommandInfo];
}

/// A `FromArgs` implementation that represents a single subcommand.
pub trait SubCommand: FromArgs {
    /// Information about the subcommand.
    const COMMAND: &'static CommandInfo;
}

impl<T: SubCommand> SubCommands for T {
    const COMMANDS: &'static [&'static CommandInfo] = &[T::COMMAND];
}

/// Information to display to the user about why a `FromArgs` construction exited early.
///
/// This can occur due to either failed parsing or a flag like `--help`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EarlyExit {
    /// The output to display to the user of the commandline tool.
    pub output: String,
    /// Status of argument parsing.
    ///
    /// `Ok` if the command was parsed successfully and the early exit is due
    /// to a flag like `--help` causing early exit with output.
    ///
    /// `Err` if the arguments were not successfully parsed.
    // TODO replace with std::process::ExitCode when stable.
    pub status: Result<(), ()>,
}

impl From<String> for EarlyExit {
    fn from(err_msg: String) -> Self {
        Self { output: err_msg, status: Err(()) }
    }
}

/// Create a `FromArgs` type from the current process's `env::args`.
///
/// This function will exit early from the current process if argument parsing
/// was unsuccessful or if information like `--help` was requested.
pub fn from_env<T: TopLevelCommand>() -> T {
    let strings: Vec<String> = std::env::args().collect();
    let strs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    T::from_args(&[strs[0]], &strs[1..]).unwrap_or_else(|early_exit| {
        println!("{}", early_exit.output);
        std::process::exit(match early_exit.status {
            Ok(()) => 0,
            Err(()) => 1,
        })
    })
}

/// Create a `FromArgs` type from the current process's `env::args`.
///
/// This special cases usages where argh is being used in an environment where cargo is
/// driving the build. We skip the second env variable.
///
/// This function will exit early from the current process if argument parsing
/// was unsuccessful or if information like `--help` was requested.
pub fn cargo_from_env<T: TopLevelCommand>() -> T {
    let strings: Vec<String> = std::env::args().collect();
    let strs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    T::from_args(&[strs[1]], &strs[2..]).unwrap_or_else(|early_exit| {
        println!("{}", early_exit.output);
        std::process::exit(match early_exit.status {
            Ok(()) => 0,
            Err(()) => 1,
        })
    })
}

/// Types which can be constructed from a single commandline value.
///
/// Any field type declared in a struct that derives `FromArgs` must implement
/// this trait. A blanket implementation exists for types implementing
/// `FromStr<Error: Display>`. Custom types can implement this trait
/// directly.
pub trait FromArgValue: Sized {
    /// Construct the type from a commandline value, returning an error string
    /// on failure.
    fn from_arg_value(value: &str) -> Result<Self, String>;
}

impl<T> FromArgValue for T
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    fn from_arg_value(value: &str) -> Result<Self, String> {
        T::from_str(value).map_err(|x| x.to_string())
    }
}

// The following items are all used by the generated code, and should not be considered part
// of this library's public API surface.

// A trait for for slots that reserve space for a value and know how to parse that value
// from a command-line `&str` argument.
//
// This trait is only implemented for the type `ParseValueSlotTy`. This indirection is
// necessary to allow abstracting over `ParseValueSlotTy` instances with different
// generic parameters.
#[doc(hidden)]
pub trait ParseValueSlot {
    fn fill_slot(&mut self, value: &str) -> Result<(), String>;
}

// The concrete type implementing the `ParseValueSlot` trait.
//
// `T` is the type to be parsed from a single string.
// `Slot` is the type of the container that can hold a value or values of type `T`.
#[doc(hidden)]
pub struct ParseValueSlotTy<Slot, T> {
    // The slot for a parsed value.
    pub slot: Slot,
    // The function to parse the value from a string
    pub parse_func: fn(&str) -> Result<T, String>,
}

// `ParseValueSlotTy<Option<T>, T>` is used as the slot for all non-repeating
// arguments, both optional and required.
impl<T> ParseValueSlot for ParseValueSlotTy<Option<T>, T> {
    fn fill_slot(&mut self, value: &str) -> Result<(), String> {
        if self.slot.is_some() {
            return Err("duplicate values provided".to_string());
        }
        self.slot = Some((self.parse_func)(value)?);
        Ok(())
    }
}

// `ParseValueSlotTy<Vec<T>, T>` is used as the slot for repeating arguments.
impl<T> ParseValueSlot for ParseValueSlotTy<Vec<T>, T> {
    fn fill_slot(&mut self, value: &str) -> Result<(), String> {
        self.slot.push((self.parse_func)(value)?);
        Ok(())
    }
}

/// A type which can be the receiver of a `Flag`.
pub trait Flag {
    /// Creates a default instance of the flag value;
    fn default() -> Self where Self: Sized;
    /// Sets the flag. This function is called when the flag is provided.
    fn set_flag(&mut self);
}

impl Flag for bool {
    fn default() -> Self {
        false
    }
    fn set_flag(&mut self) {
        *self = true;
    }
}

macro_rules! impl_flag_for_integers {
    ($($ty:ty,)*) => {
        $(
            impl Flag for $ty {
                fn default() -> Self {
                    0
                }
                fn set_flag(&mut self) {
                    *self = self.saturating_add(1);
                }
            }
        )*
    }
}

impl_flag_for_integers![
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
];

// `--` or `-` options, including a mutable reference to their value.
#[doc(hidden)]
pub enum CmdOption<'a> {
    // A flag which is set to `true` when provided.
    Flag(&'a mut dyn Flag),
    // A value which is parsed from the string following the `--` argument,
    // e.g. `--foo bar`.
    Value(&'a mut dyn ParseValueSlot),
}

#[doc(hidden)]
pub fn unrecognized_argument(x: &str) -> String {
    ["Unrecognized argument: ", x, "\n"].concat()
}

// A sentinel value that indicates that there is no
// output table mapping for the given flag.
// This is used for arguments like `--verbose` and `--quiet`
// that must be silently accepted if the `argh` user hasn't
// specified their behavior explicitly.
#[doc(hidden)]
pub const OUTPUT_TABLE_NONE: usize = std::usize::MAX;

/// Parse a commandline option.
///
/// `arg`: the current option argument being parsed (e.g. `--foo`).
/// `remaining_args`: the remaining command line arguments. This slice
/// will be advanced forwards if the option takes a value argument.
/// `output_table`: the storage for output data.
/// `arg_to_input`: a mapping from option string literals to the entry
/// in the output table. This may contain multiple entries mapping to
/// the same location in the table if both a short and long version
/// of the option exist (`-z` and `--zoo`).
#[doc(hidden)]
pub fn parse_option(
    arg: &str,
    remaining_args: &mut &[&str],
    output_table: &mut [CmdOption<'_>],
    arg_to_output: &[(&str, usize)],
) -> Result<(), String> {
    let pos = arg_to_output
        .iter()
        .find_map(|&(name, pos)| if name == arg { Some(pos) } else { None })
        .ok_or_else(|| unrecognized_argument(arg))?;

    if pos == OUTPUT_TABLE_NONE {
        return Ok(());
    }

    match &mut output_table[pos] {
        CmdOption::Flag(b) => b.set_flag(),
        CmdOption::Value(pvs) => {
            let value = remaining_args.get(0).ok_or_else(|| {
                ["No value provided for option '", arg, "'.\n"].concat()
            })?;
            *remaining_args = &remaining_args[1..];
            pvs.fill_slot(value).map_err(|s| {
                ["Error parsing option '", arg, "' with value '", value, "': ", &s, "\n"].concat()
            })?;
        }
    }

    Ok(())
}

/// Parse a positional argument.
///
/// arg: the argument supplied by the user
/// positional: a tuple containing slot to parse into and the name of the argument
#[doc(hidden)]
pub fn parse_positional(
    arg: &str,
    positional: &mut (&mut dyn ParseValueSlot, &'static str),
) -> Result<(), String> {
    let (slot, name) = positional;
    slot.fill_slot(arg).map_err(|s| {
        ["Error parsing positional argument '", name, "' with value '", arg, ": ", &s].concat()
    })
}

// Prepend `help` to a list of arguments.
// This is used to pass the `help` argument on to subcommands.
#[doc(hidden)]
pub fn prepend_help<'a>(args: &[&'a str]) -> Vec<&'a str> {
    [&["help"], args].concat()
}

#[doc(hidden)]
pub fn print_subcommands(commands: &[&CommandInfo]) -> String {
    let mut out = String::new();
    for cmd in commands {
        argh_shared::write_description(&mut out, cmd);
    }
    out
}

#[doc(hidden)]
pub fn expected_subcommand(commands: &[&str]) -> String {
    ["Expected one of the following subcommands: ", &commands.join(", "), "\n"].concat()
}

#[doc(hidden)]
pub fn unrecognized_arg(arg: &str) -> String {
    ["Unrecognized argument: ", arg, "\n"].concat()
}

// An error string builder to report missing required options and subcommands.
#[doc(hidden)]
#[derive(Default)]
pub struct MissingRequirements {
    options: Vec<&'static str>,
    subcommands: Option<&'static [&'static CommandInfo]>,
    positional_args: Vec<&'static str>,
}

const NEWLINE_INDENT: &str = "\n    ";

impl MissingRequirements {
    // Add a missing required option.
    #[doc(hidden)]
    pub fn missing_option(&mut self, name: &'static str) {
        self.options.push(name)
    }

    // Add a missing required subcommand.
    #[doc(hidden)]
    pub fn missing_subcommands(&mut self, commands: &'static [&'static CommandInfo]) {
        self.subcommands = Some(commands);
    }

    // Add a missing positional argument.
    #[doc(hidden)]
    pub fn missing_positional_arg(&mut self, name: &'static str) {
        self.positional_args.push(name)
    }

    // If any missing options or subcommands were provided, returns an error string
    // describing the missing args.
    #[doc(hidden)]
    pub fn err_on_any(&self) -> Result<(), String> {
        if self.options.is_empty()
            && self.subcommands.is_none()
            && self.positional_args.is_empty()
        {
            return Ok(());
        }

        let mut output = String::new();

        if !self.positional_args.is_empty() {
            output.push_str("Required positional arguments not provided:");
            for arg in &self.positional_args {
                output.push_str(NEWLINE_INDENT);
                output.push_str(arg);
            }
        }

        if !self.options.is_empty() {
            output.push_str("Required options not provided:");
            for option in &self.options {
                output.push_str(NEWLINE_INDENT);
                output.push_str(option);
            }
        }

        if let Some(missing_subcommands) = self.subcommands {
            if !self.options.is_empty() {
                output.push_str("\n");
            }
            output.push_str("One of the following subcommands must be present:");
            output.push_str(NEWLINE_INDENT);
            output.push_str("help");
            for subcommand in missing_subcommands {
                output.push_str(NEWLINE_INDENT);
                output.push_str(subcommand.name);
            }
        }

        output.push('\n');

        Err(output)
    }
}
