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
//! let up: GoUp = argh::from_env();
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
//!   --help, help      display usage information
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
//! `#[argh(default = "<your_code_here>")]` attribute, and in this case an
//! option is treated as optional.
//!
//! ```rust
//! use argh::FromArgs;
//!
//! fn default_height() -> usize {
//!     5
//! }
//!
//! #[derive(FromArgs)]
//! /// Reach new heights.
//! #[argh(help_triggers("-h", "--help", "help"))]
//! struct GoUp {
//!     /// an optional nickname for the pilot
//!     #[argh(option)]
//!     pilot_nickname: Option<String>,
//!
//!     /// an optional height
//!     #[argh(option, default = "default_height()")]
//!     height: usize,
//!
//!     /// an optional direction which is "up" by default
//!     #[argh(option, default = "String::from(\"only up\")")]
//!     direction: String,
//! }
//!
//! fn main() {
//!     let up: GoUp = argh::from_env();
//! }
//! ```
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
//! `Option` or `Vec` to indicate an optional or repeating positional argument.
//!
//! If your final positional argument has the `greedy` option on it, it will consume
//! any arguments after it as if a `--` were placed before the first argument to
//! match the greedy positional:
//!
//! ```rust
//! use argh::FromArgs;
//! #[derive(FromArgs, PartialEq, Debug)]
//! /// A command with a greedy positional argument at the end.
//! struct WithGreedyPositional {
//!     /// some stuff
//!     #[argh(option)]
//!     stuff: Option<String>,
//!     #[argh(positional, greedy)]
//!     all_the_rest: Vec<String>,
//! }
//! ```
//!
//! Now if you pass `--stuff Something` after a positional argument, it will
//! be consumed by `all_the_rest` instead of setting the `stuff` field.
//!
//! Note that `all_the_rest` won't be listed as a positional argument in the
//! long text part of help output (and it will be listed at the end of the usage
//! line as `[all_the_rest...]`), and it's up to the caller to append any
//! extra help output for the meaning of the captured arguments. This is to
//! enable situations where some amount of argument processing needs to happen
//! before the rest of the arguments can be interpreted, and shouldn't be used
//! for regular use as it might be confusing.
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
//!
//! You can also discover subcommands dynamically at runtime. To do this,
//! declare subcommands as usual and add a variant to the enum with the
//! `dynamic` attribute. Instead of deriving `FromArgs`, the value inside the
//! dynamic variant should implement `DynamicSubCommand`.
//!
//! ```rust
//! # use argh::CommandInfo;
//! # use argh::DynamicSubCommand;
//! # use argh::EarlyExit;
//! # use argh::FromArgs;
//! # use once_cell::sync::OnceCell;
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
//!     Normal(NormalSubCommand),
//!     #[argh(dynamic)]
//!     Dynamic(Dynamic),
//! }
//!
//! #[derive(FromArgs, PartialEq, Debug)]
//! /// Normal subcommand.
//! #[argh(subcommand, name = "normal")]
//! struct NormalSubCommand {
//!     #[argh(option)]
//!     /// how many x
//!     x: usize,
//! }
//!
//! /// Dynamic subcommand.
//! #[derive(PartialEq, Debug)]
//! struct Dynamic {
//!     name: String
//! }
//!
//! impl DynamicSubCommand for Dynamic {
//!     fn commands() -> &'static [&'static CommandInfo] {
//!         static RET: OnceCell<Vec<&'static CommandInfo>> = OnceCell::new();
//!         RET.get_or_init(|| {
//!             let mut commands = Vec::new();
//!
//!             // argh needs the `CommandInfo` structs we generate to be valid
//!             // for the static lifetime. We can allocate the structures on
//!             // the heap with `Box::new` and use `Box::leak` to get a static
//!             // reference to them. We could also just use a constant
//!             // reference, but only because this is a synthetic example; the
//!             // point of using dynamic commands is to have commands you
//!             // don't know about until runtime!
//!             commands.push(&*Box::leak(Box::new(CommandInfo {
//!                 name: "dynamic_command",
//!                 description: "A dynamic command",
//!             })));
//!
//!             commands
//!         })
//!     }
//!
//!     fn try_redact_arg_values(
//!         command_name: &[&str],
//!         args: &[&str],
//!     ) -> Option<Result<Vec<String>, EarlyExit>> {
//!         for command in Self::commands() {
//!             if command_name.last() == Some(&command.name) {
//!                 // Process arguments and redact values here.
//!                 if !args.is_empty() {
//!                     return Some(Err("Our example dynamic command never takes arguments!"
//!                                     .to_string().into()));
//!                 }
//!                 return Some(Ok(Vec::new()))
//!             }
//!         }
//!         None
//!     }
//!
//!     fn try_from_args(command_name: &[&str], args: &[&str]) -> Option<Result<Self, EarlyExit>> {
//!         for command in Self::commands() {
//!             if command_name.last() == Some(&command.name) {
//!                 if !args.is_empty() {
//!                     return Some(Err("Our example dynamic command never takes arguments!"
//!                                     .to_string().into()));
//!                 }
//!                 return Some(Ok(Dynamic { name: command.name.to_string() }))
//!             }
//!         }
//!         None
//!     }
//! }
//! ```
//!
//! Programs that are run from an environment such as cargo may find it
//! useful to have positional arguments present in the structure but
//! omitted from the usage output. This can be accomplished by adding
//! the `hidden_help` attribute to that argument:
//!
//! ```rust
//! # use argh::FromArgs;
//!
//! #[derive(FromArgs)]
//! /// Cargo arguments
//! struct CargoArgs {
//!     // Cargo puts the command name invoked into the first argument,
//!     // so we don't want this argument to show up in the usage text.
//!     #[argh(positional, hidden_help)]
//!     command: String,
//!     /// an option used for internal debugging
//!     #[argh(option, hidden_help)]
//!     internal_debugging: String,
//!     #[argh(positional)]
//!     real_first_arg: String,
//! }
//! ```

#![deny(missing_docs)]

use std::str::FromStr;

pub use argh_derive::{ArgsInfo, FromArgs};

/// Information about a particular command used for output.
pub type CommandInfo = argh_shared::CommandInfo<'static>;

/// Information about the command including the options and arguments.
pub type CommandInfoWithArgs = argh_shared::CommandInfoWithArgs<'static>;

/// Information about a subcommand.
pub type SubCommandInfo = argh_shared::SubCommandInfo<'static>;

pub use argh_shared::{ErrorCodeInfo, FlagInfo, FlagInfoKind, Optionality, PositionalInfo};

/// Structured information about the command line arguments.
pub trait ArgsInfo {
    /// Returns the argument info.
    fn get_args_info() -> CommandInfoWithArgs;

    /// Returns the list of subcommands
    fn get_subcommands() -> Vec<SubCommandInfo> {
        Self::get_args_info().commands
    }
}

/// Types which can be constructed from a set of commandline arguments.
pub trait FromArgs: Sized {
    /// Construct the type from an input set of arguments.
    ///
    /// The first argument `command_name` is the identifier for the current command. In most cases,
    /// users should only pass in a single item for the command name, which typically comes from
    /// the first item from `std::env::args()`. Implementations however should append the
    /// subcommand name in when recursively calling [FromArgs::from_args] for subcommands. This
    /// allows `argh` to generate correct subcommand help strings.
    ///
    /// The second argument `args` is the rest of the command line arguments.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use argh::FromArgs;
    ///
    /// /// Command to manage a classroom.
    /// #[derive(Debug, PartialEq, FromArgs)]
    /// struct ClassroomCmd {
    ///     #[argh(subcommand)]
    ///     subcommands: Subcommands,
    /// }
    ///
    /// #[derive(Debug, PartialEq, FromArgs)]
    /// #[argh(subcommand)]
    /// enum Subcommands {
    ///     List(ListCmd),
    ///     Add(AddCmd),
    /// }
    ///
    /// /// list all the classes.
    /// #[derive(Debug, PartialEq, FromArgs)]
    /// #[argh(subcommand, name = "list")]
    /// struct ListCmd {
    ///     /// list classes for only this teacher.
    ///     #[argh(option)]
    ///     teacher_name: Option<String>,
    /// }
    ///
    /// /// add students to a class.
    /// #[derive(Debug, PartialEq, FromArgs)]
    /// #[argh(subcommand, name = "add")]
    /// struct AddCmd {
    ///     /// the name of the class's teacher.
    ///     #[argh(option)]
    ///     teacher_name: String,
    ///
    ///     /// the name of the class.
    ///     #[argh(positional)]
    ///     class_name: String,
    /// }
    ///
    /// let args = ClassroomCmd::from_args(
    ///     &["classroom"],
    ///     &["list", "--teacher-name", "Smith"],
    /// ).unwrap();
    /// assert_eq!(
    ///    args,
    ///     ClassroomCmd {
    ///         subcommands: Subcommands::List(ListCmd {
    ///             teacher_name: Some("Smith".to_string()),
    ///         })
    ///     },
    /// );
    ///
    /// // Help returns an error, but internally returns an `Ok` status.
    /// let early_exit = ClassroomCmd::from_args(
    ///     &["classroom"],
    ///     &["help"],
    /// ).unwrap_err();
    /// assert_eq!(
    ///     early_exit,
    ///     argh::EarlyExit {
    ///        output: r#"Usage: classroom <command> [<args>]
    ///
    /// Command to manage a classroom.
    ///
    /// Options:
    ///   --help, help      display usage information
    ///
    /// Commands:
    ///   list              list all the classes.
    ///   add               add students to a class.
    /// "#.to_string(),
    ///        status: Ok(()),
    ///     },
    /// );
    ///
    /// // Help works with subcommands.
    /// let early_exit = ClassroomCmd::from_args(
    ///     &["classroom"],
    ///     &["list", "help"],
    /// ).unwrap_err();
    /// assert_eq!(
    ///     early_exit,
    ///     argh::EarlyExit {
    ///        output: r#"Usage: classroom list [--teacher-name <teacher-name>]
    ///
    /// list all the classes.
    ///
    /// Options:
    ///   --teacher-name    list classes for only this teacher.
    ///   --help, help      display usage information
    /// "#.to_string(),
    ///        status: Ok(()),
    ///     },
    /// );
    ///
    /// // Incorrect arguments will error out.
    /// let err = ClassroomCmd::from_args(
    ///     &["classroom"],
    ///     &["lisp"],
    /// ).unwrap_err();
    /// assert_eq!(
    ///    err,
    ///    argh::EarlyExit {
    ///        output: "Unrecognized argument: lisp\n".to_string(),
    ///        status: Err(()),
    ///     },
    /// );
    /// ```
    fn from_args(command_name: &[&str], args: &[&str]) -> Result<Self, EarlyExit>;

    /// Get a String with just the argument names, e.g., options, flags, subcommands, etc, but
    /// without the values of the options and arguments. This can be useful as a means to capture
    /// anonymous usage statistics without revealing the content entered by the end user.
    ///
    /// The first argument `command_name` is the identifier for the current command. In most cases,
    /// users should only pass in a single item for the command name, which typically comes from
    /// the first item from `std::env::args()`. Implementations however should append the
    /// subcommand name in when recursively calling [FromArgs::from_args] for subcommands. This
    /// allows `argh` to generate correct subcommand help strings.
    ///
    /// The second argument `args` is the rest of the command line arguments.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use argh::FromArgs;
    ///
    /// /// Command to manage a classroom.
    /// #[derive(FromArgs)]
    /// struct ClassroomCmd {
    ///     #[argh(subcommand)]
    ///     subcommands: Subcommands,
    /// }
    ///
    /// #[derive(FromArgs)]
    /// #[argh(subcommand)]
    /// enum Subcommands {
    ///     List(ListCmd),
    ///     Add(AddCmd),
    /// }
    ///
    /// /// list all the classes.
    /// #[derive(FromArgs)]
    /// #[argh(subcommand, name = "list")]
    /// struct ListCmd {
    ///     /// list classes for only this teacher.
    ///     #[argh(option)]
    ///     teacher_name: Option<String>,
    /// }
    ///
    /// /// add students to a class.
    /// #[derive(FromArgs)]
    /// #[argh(subcommand, name = "add")]
    /// struct AddCmd {
    ///     /// the name of the class's teacher.
    ///     #[argh(option)]
    ///     teacher_name: String,
    ///
    ///     /// has the class started yet?
    ///     #[argh(switch)]
    ///     started: bool,
    ///
    ///     /// the name of the class.
    ///     #[argh(positional)]
    ///     class_name: String,
    ///
    ///     /// the student names.
    ///     #[argh(positional)]
    ///     students: Vec<String>,
    /// }
    ///
    /// let args = ClassroomCmd::redact_arg_values(
    ///     &["classroom"],
    ///     &["list"],
    /// ).unwrap();
    /// assert_eq!(
    ///     args,
    ///     &[
    ///         "classroom",
    ///         "list",
    ///     ],
    /// );
    ///
    /// let args = ClassroomCmd::redact_arg_values(
    ///     &["classroom"],
    ///     &["list", "--teacher-name", "Smith"],
    /// ).unwrap();
    /// assert_eq!(
    ///    args,
    ///    &[
    ///         "classroom",
    ///         "list",
    ///         "--teacher-name",
    ///     ],
    /// );
    ///
    /// let args = ClassroomCmd::redact_arg_values(
    ///     &["classroom"],
    ///     &["add", "--teacher-name", "Smith", "--started", "Math", "Abe", "Sung"],
    /// ).unwrap();
    /// assert_eq!(
    ///     args,
    ///     &[
    ///         "classroom",
    ///         "add",
    ///         "--teacher-name",
    ///         "--started",
    ///         "class_name",
    ///         "students",
    ///         "students",
    ///     ],
    /// );
    ///
    /// // `ClassroomCmd::redact_arg_values` will error out if passed invalid arguments.
    /// assert_eq!(
    ///     ClassroomCmd::redact_arg_values(&["classroom"], &["add", "--teacher-name"]),
    ///     Err(argh::EarlyExit {
    ///         output: "No value provided for option '--teacher-name'.\n".into(),
    ///         status: Err(()),
    ///     }),
    /// );
    ///
    /// // `ClassroomCmd::redact_arg_values` will generate help messages.
    /// assert_eq!(
    ///     ClassroomCmd::redact_arg_values(&["classroom"], &["help"]),
    ///     Err(argh::EarlyExit {
    ///         output: r#"Usage: classroom <command> [<args>]
    ///
    /// Command to manage a classroom.
    ///
    /// Options:
    ///   --help, help      display usage information
    ///
    /// Commands:
    ///   list              list all the classes.
    ///   add               add students to a class.
    /// "#.to_string(),
    ///         status: Ok(()),
    ///     }),
    /// );
    /// ```
    fn redact_arg_values(_command_name: &[&str], _args: &[&str]) -> Result<Vec<String>, EarlyExit> {
        Ok(vec!["<<REDACTED>>".into()])
    }
}

/// A top-level `FromArgs` implementation that is not a subcommand.
pub trait TopLevelCommand: FromArgs {}

/// A `FromArgs` implementation that can parse into one or more subcommands.
pub trait SubCommands: FromArgs {
    /// Info for the commands.
    const COMMANDS: &'static [&'static CommandInfo];

    /// Get a list of commands that are discovered at runtime.
    fn dynamic_commands() -> &'static [&'static CommandInfo] {
        &[]
    }
}

/// A `FromArgs` implementation that represents a single subcommand.
pub trait SubCommand: FromArgs {
    /// Information about the subcommand.
    const COMMAND: &'static CommandInfo;
}

impl<T: SubCommand> SubCommands for T {
    const COMMANDS: &'static [&'static CommandInfo] = &[T::COMMAND];
}

/// Trait implemented by values returned from a dynamic subcommand handler.
pub trait DynamicSubCommand: Sized {
    /// Info about supported subcommands.
    fn commands() -> &'static [&'static CommandInfo];

    /// Perform the function of `FromArgs::redact_arg_values` for this dynamic
    /// command.
    ///
    /// The full list of subcommands, ending with the subcommand that should be
    /// dynamically recognized, is passed in `command_name`. If the command
    /// passed is not recognized, this function should return `None`. Otherwise
    /// it should return `Some`, and the value within the `Some` has the same
    /// semantics as the return of `FromArgs::redact_arg_values`.
    fn try_redact_arg_values(
        command_name: &[&str],
        args: &[&str],
    ) -> Option<Result<Vec<String>, EarlyExit>>;

    /// Perform the function of `FromArgs::from_args` for this dynamic command.
    ///
    /// The full list of subcommands, ending with the subcommand that should be
    /// dynamically recognized, is passed in `command_name`. If the command
    /// passed is not recognized, this function should return `None`. Otherwise
    /// it should return `Some`, and the value within the `Some` has the same
    /// semantics as the return of `FromArgs::from_args`.
    fn try_from_args(command_name: &[&str], args: &[&str]) -> Option<Result<Self, EarlyExit>>;
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

/// Extract the base cmd from a path
fn cmd<'a>(default: &'a str, path: &'a str) -> &'a str {
    std::path::Path::new(path).file_name().and_then(|s| s.to_str()).unwrap_or(default)
}

/// Create a `FromArgs` type from the current process's `env::args`.
///
/// This function will exit early from the current process if argument parsing
/// was unsuccessful or if information like `--help` was requested. Error messages will be printed
/// to stderr, and `--help` output to stdout.
pub fn from_env<T: TopLevelCommand>() -> T {
    let strings: Vec<String> = std::env::args_os()
        .map(|s| s.into_string())
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|arg| {
            eprintln!("Invalid utf8: {}", arg.to_string_lossy());
            std::process::exit(1)
        });

    if strings.is_empty() {
        eprintln!("No program name, argv is empty");
        std::process::exit(1)
    }

    let cmd = cmd(&strings[0], &strings[0]);
    let strs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    T::from_args(&[cmd], &strs[1..]).unwrap_or_else(|early_exit| {
        std::process::exit(match early_exit.status {
            Ok(()) => {
                println!("{}", early_exit.output);
                0
            }
            Err(()) => {
                eprintln!("{}\nRun {} --help for more information.", early_exit.output, cmd);
                1
            }
        })
    })
}

/// Create a `FromArgs` type from the current process's `env::args`.
///
/// This special cases usages where argh is being used in an environment where cargo is
/// driving the build. We skip the second env argument.
///
/// This function will exit early from the current process if argument parsing
/// was unsuccessful or if information like `--help` was requested. Error messages will be printed
/// to stderr, and `--help` output to stdout.
pub fn cargo_from_env<T: TopLevelCommand>() -> T {
    let strings: Vec<String> = std::env::args().collect();
    let cmd = cmd(&strings[1], &strings[1]);
    let strs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    T::from_args(&[cmd], &strs[2..]).unwrap_or_else(|early_exit| {
        std::process::exit(match early_exit.status {
            Ok(()) => {
                println!("{}", early_exit.output);
                0
            }
            Err(()) => {
                eprintln!("{}\nRun --help for more information.", early_exit.output);
                1
            }
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

#[doc(hidden)]
pub trait ParseFlag {
    fn set_flag(&mut self, arg: &str);
}

impl<T: Flag> ParseFlag for T {
    fn set_flag(&mut self, _arg: &str) {
        <T as Flag>::set_flag(self);
    }
}

#[doc(hidden)]
pub struct RedactFlag {
    pub slot: Option<String>,
}

impl ParseFlag for RedactFlag {
    fn set_flag(&mut self, arg: &str) {
        self.slot = Some(arg.to_string());
    }
}

// A trait for for slots that reserve space for a value and know how to parse that value
// from a command-line `&str` argument.
//
// This trait is only implemented for the type `ParseValueSlotTy`. This indirection is
// necessary to allow abstracting over `ParseValueSlotTy` instances with different
// generic parameters.
#[doc(hidden)]
pub trait ParseValueSlot {
    fn fill_slot(&mut self, arg: &str, value: &str) -> Result<(), String>;
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
    pub parse_func: fn(&str, &str) -> Result<T, String>,
}

// `ParseValueSlotTy<Option<T>, T>` is used as the slot for all non-repeating
// arguments, both optional and required.
impl<T> ParseValueSlot for ParseValueSlotTy<Option<T>, T> {
    fn fill_slot(&mut self, arg: &str, value: &str) -> Result<(), String> {
        if self.slot.is_some() {
            return Err("duplicate values provided".to_string());
        }
        self.slot = Some((self.parse_func)(arg, value)?);
        Ok(())
    }
}

// `ParseValueSlotTy<Vec<T>, T>` is used as the slot for repeating arguments.
impl<T> ParseValueSlot for ParseValueSlotTy<Vec<T>, T> {
    fn fill_slot(&mut self, arg: &str, value: &str) -> Result<(), String> {
        self.slot.push((self.parse_func)(arg, value)?);
        Ok(())
    }
}

// `ParseValueSlotTy<Option<Vec<T>>, T>` is used as the slot for optional repeating arguments.
impl<T> ParseValueSlot for ParseValueSlotTy<Option<Vec<T>>, T> {
    fn fill_slot(&mut self, arg: &str, value: &str) -> Result<(), String> {
        self.slot.get_or_insert_with(Vec::new).push((self.parse_func)(arg, value)?);
        Ok(())
    }
}

/// A type which can be the receiver of a `Flag`.
pub trait Flag {
    /// Creates a default instance of the flag value;
    fn default() -> Self
    where
        Self: Sized;

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

impl Flag for Option<bool> {
    fn default() -> Self {
        None
    }

    fn set_flag(&mut self) {
        *self = Some(true);
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

impl_flag_for_integers![u8, u16, u32, u64, u128, i8, i16, i32, i64, i128,];

/// This function implements argument parsing for structs.
///
/// `cmd_name`: The identifier for the current command.
/// `args`: The command line arguments.
/// `parse_options`: Helper to parse optional arguments.
/// `parse_positionals`: Helper to parse positional arguments.
/// `parse_subcommand`: Helper to parse a subcommand.
/// `help_func`: Generate a help message.
#[doc(hidden)]
pub fn parse_struct_args(
    cmd_name: &[&str],
    args: &[&str],
    mut parse_options: ParseStructOptions<'_>,
    mut parse_positionals: ParseStructPositionals<'_>,
    mut parse_subcommand: Option<ParseStructSubCommand<'_>>,
    help_func: &dyn Fn() -> String,
) -> Result<(), EarlyExit> {
    let mut help = false;
    let mut remaining_args = args;
    let mut positional_index = 0;
    let mut options_ended = false;

    'parse_args: while let Some(&next_arg) = remaining_args.first() {
        remaining_args = &remaining_args[1..];
        if (parse_options.help_triggers.contains(&next_arg)) && !options_ended {
            help = true;
            continue;
        }

        if next_arg.starts_with('-') && !options_ended {
            if next_arg == "--" {
                options_ended = true;
                continue;
            }

            if help {
                return Err("Trailing arguments are not allowed after `help`.".to_string().into());
            }

            parse_options.parse(next_arg, &mut remaining_args)?;
            continue;
        }

        if let Some(ref mut parse_subcommand) = parse_subcommand {
            if parse_subcommand.parse(help, cmd_name, next_arg, remaining_args)? {
                // Unset `help`, since we handled it in the subcommand
                help = false;
                break 'parse_args;
            }
        }

        options_ended |= parse_positionals.parse(&mut positional_index, next_arg)?;
    }

    if help {
        Err(EarlyExit { output: help_func(), status: Ok(()) })
    } else {
        Ok(())
    }
}

#[doc(hidden)]
pub struct ParseStructOptions<'a> {
    /// A mapping from option string literals to the entry
    /// in the output table. This may contain multiple entries mapping to
    /// the same location in the table if both a short and long version
    /// of the option exist (`-z` and `--zoo`).
    pub arg_to_slot: &'static [(&'static str, usize)],

    /// The storage for argument output data.
    pub slots: &'a mut [ParseStructOption<'a>],

    /// help triggers is a list of strings that trigger printing of help
    pub help_triggers: &'a [&'a str],
}

impl<'a> ParseStructOptions<'a> {
    /// Parse a commandline option.
    ///
    /// `arg`: the current option argument being parsed (e.g. `--foo`).
    /// `remaining_args`: the remaining command line arguments. This slice
    /// will be advanced forwards if the option takes a value argument.
    fn parse(&mut self, arg: &str, remaining_args: &mut &[&str]) -> Result<(), String> {
        let pos = self
            .arg_to_slot
            .iter()
            .find_map(|&(name, pos)| if name == arg { Some(pos) } else { None })
            .ok_or_else(|| unrecognized_argument(arg))?;

        match self.slots[pos] {
            ParseStructOption::Flag(ref mut b) => b.set_flag(arg),
            ParseStructOption::Value(ref mut pvs) => {
                let value = remaining_args
                    .first()
                    .ok_or_else(|| ["No value provided for option '", arg, "'.\n"].concat())?;
                *remaining_args = &remaining_args[1..];
                pvs.fill_slot(arg, value).map_err(|s| {
                    ["Error parsing option '", arg, "' with value '", value, "': ", &s, "\n"]
                        .concat()
                })?;
            }
        }

        Ok(())
    }
}

fn unrecognized_argument(x: &str) -> String {
    ["Unrecognized argument: ", x, "\n"].concat()
}

// `--` or `-` options, including a mutable reference to their value.
#[doc(hidden)]
pub enum ParseStructOption<'a> {
    // A flag which is set to `true` when provided.
    Flag(&'a mut dyn ParseFlag),
    // A value which is parsed from the string following the `--` argument,
    // e.g. `--foo bar`.
    Value(&'a mut dyn ParseValueSlot),
}

#[doc(hidden)]
pub struct ParseStructPositionals<'a> {
    pub positionals: &'a mut [ParseStructPositional<'a>],
    pub last_is_repeating: bool,
    pub last_is_greedy: bool,
}

impl<'a> ParseStructPositionals<'a> {
    /// Parse the next positional argument.
    ///
    /// `arg`: the argument supplied by the user.
    ///
    /// Returns true if non-positional argument parsing should stop
    /// after this one.
    fn parse(&mut self, index: &mut usize, arg: &str) -> Result<bool, EarlyExit> {
        if *index < self.positionals.len() {
            self.positionals[*index].parse(arg)?;

            if self.last_is_repeating && *index == self.positionals.len() - 1 {
                // Don't increment position if we're at the last arg
                // *and* the last arg is repeating. If it's also remainder,
                // halt non-option processing after this.
                Ok(self.last_is_greedy)
            } else {
                // If it is repeating, though, increment the index and continue
                // processing options.
                *index += 1;
                Ok(false)
            }
        } else {
            Err(EarlyExit { output: unrecognized_arg(arg), status: Err(()) })
        }
    }
}

#[doc(hidden)]
pub struct ParseStructPositional<'a> {
    // The positional's name
    pub name: &'static str,

    // The function to parse the positional.
    pub slot: &'a mut dyn ParseValueSlot,
}

impl<'a> ParseStructPositional<'a> {
    /// Parse a positional argument.
    ///
    /// `arg`: the argument supplied by the user.
    fn parse(&mut self, arg: &str) -> Result<(), EarlyExit> {
        self.slot.fill_slot("", arg).map_err(|s| {
            [
                "Error parsing positional argument '",
                self.name,
                "' with value '",
                arg,
                "': ",
                &s,
                "\n",
            ]
            .concat()
            .into()
        })
    }
}

// A type to simplify parsing struct subcommands.
//
// This indirection is necessary to allow abstracting over `FromArgs` instances with different
// generic parameters.
#[doc(hidden)]
pub struct ParseStructSubCommand<'a> {
    // The subcommand commands
    pub subcommands: &'static [&'static CommandInfo],

    pub dynamic_subcommands: &'a [&'static CommandInfo],

    // The function to parse the subcommand arguments.
    #[allow(clippy::type_complexity)]
    pub parse_func: &'a mut dyn FnMut(&[&str], &[&str]) -> Result<(), EarlyExit>,
}

impl<'a> ParseStructSubCommand<'a> {
    fn parse(
        &mut self,
        help: bool,
        cmd_name: &[&str],
        arg: &str,
        remaining_args: &[&str],
    ) -> Result<bool, EarlyExit> {
        for subcommand in self.subcommands.iter().chain(self.dynamic_subcommands.iter()) {
            if subcommand.name == arg {
                let mut command = cmd_name.to_owned();
                command.push(subcommand.name);
                let prepended_help;
                let remaining_args = if help {
                    prepended_help = prepend_help(remaining_args);
                    &prepended_help
                } else {
                    remaining_args
                };

                (self.parse_func)(&command, remaining_args)?;

                return Ok(true);
            }
        }

        Ok(false)
    }
}

// Prepend `help` to a list of arguments.
// This is used to pass the `help` argument on to subcommands.
fn prepend_help<'a>(args: &[&'a str]) -> Vec<&'a str> {
    [&["help"], args].concat()
}

#[doc(hidden)]
pub fn print_subcommands<'a>(commands: impl Iterator<Item = &'a CommandInfo>) -> String {
    let mut out = String::new();
    for cmd in commands {
        argh_shared::write_description(&mut out, cmd);
    }
    out
}

fn unrecognized_arg(arg: &str) -> String {
    ["Unrecognized argument: ", arg, "\n"].concat()
}

// An error string builder to report missing required options and subcommands.
#[doc(hidden)]
#[derive(Default)]
pub struct MissingRequirements {
    options: Vec<&'static str>,
    subcommands: Option<Vec<&'static CommandInfo>>,
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
    pub fn missing_subcommands(&mut self, commands: impl Iterator<Item = &'static CommandInfo>) {
        self.subcommands = Some(commands.collect());
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
        if self.options.is_empty() && self.subcommands.is_none() && self.positional_args.is_empty()
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
            if !self.positional_args.is_empty() {
                output.push('\n');
            }
            output.push_str("Required options not provided:");
            for option in &self.options {
                output.push_str(NEWLINE_INDENT);
                output.push_str(option);
            }
        }

        if let Some(missing_subcommands) = &self.subcommands {
            if !self.options.is_empty() {
                output.push('\n');
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cmd_extraction() {
        let expected = "test_cmd";
        let path = format!("/tmp/{}", expected);
        let cmd = cmd(&path, &path);
        assert_eq!(expected, cmd);
    }
}
