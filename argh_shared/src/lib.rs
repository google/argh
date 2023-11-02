// Copyright (c) 2020 Google LLC All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//! Shared functionality between argh_derive and the argh runtime.
//!
//! This library is intended only for internal use by these two crates.

/// Information about a particular command used for output.
pub struct CommandInfo<'a> {
    /// The name of the command.
    pub name: &'a str,
    /// A short description of the command's functionality.
    pub description: &'a str,
}

/// Information about the command line arguments for a given command.
#[derive(Debug, Default, PartialEq, Eq, Clone, serde::Serialize)]
pub struct CommandInfoWithArgs<'a> {
    /// The name of the command.
    pub name: &'a str,
    /// A short description of the command's functionality.
    pub description: &'a str,
    /// Examples of usage
    pub examples: &'a [&'a str],
    /// Flags
    pub flags: &'a [FlagInfo<'a>],
    /// Notes about usage
    pub notes: &'a [&'a str],
    /// The subcommands.
    pub commands: Vec<SubCommandInfo<'a>>,
    /// Positional args
    pub positionals: &'a [PositionalInfo<'a>],
    /// Error code information
    pub error_codes: &'a [ErrorCodeInfo<'a>],
}

/// Information about a documented error code.
#[derive(Debug, PartialEq, Eq, serde::Serialize)]
pub struct ErrorCodeInfo<'a> {
    /// The code value.
    pub code: i32,
    /// Short description about what this code indicates.
    pub description: &'a str,
}

/// Information about positional arguments
#[derive(Debug, PartialEq, Eq, serde::Serialize)]
pub struct PositionalInfo<'a> {
    /// Name of the argument.
    pub name: &'a str,
    /// Description of the argument.
    pub description: &'a str,
    /// Optionality of the argument.
    pub optionality: Optionality,
    /// Visibility in the help for this argument.
    /// `false` indicates this argument will not appear
    /// in the help message.
    pub hidden: bool,
}

/// Information about a subcommand.
/// Dynamic subcommands do not implement
/// get_args_info(), so the command field
/// only contains the name and description.
#[derive(Debug, Default, PartialEq, Eq, Clone, serde::Serialize)]
pub struct SubCommandInfo<'a> {
    /// The subcommand name.
    pub name: &'a str,
    /// The information about the subcommand.
    pub command: CommandInfoWithArgs<'a>,
}

/// Information about a flag or option.
#[derive(Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct FlagInfo<'a> {
    /// The kind of flag.
    pub kind: FlagInfoKind<'a>,
    /// The optionality of the flag.
    pub optionality: Optionality,
    /// The long string of the flag.
    pub long: &'a str,
    /// The single character short indicator
    /// for this flag.
    pub short: Option<char>,
    /// The description of the flag.
    pub description: &'a str,
    /// Visibility in the help for this argument.
    /// `false` indicates this argument will not appear
    /// in the help message.
    pub hidden: bool,
}

/// The kind of flags.
#[derive(Debug, Default, PartialEq, Eq, serde::Serialize)]
pub enum FlagInfoKind<'a> {
    /// switch represents a boolean flag,
    #[default]
    Switch,
    /// option is a flag that also has an associated
    /// value. This value is named `arg_name`.
    Option { arg_name: &'a str },
}

/// The optionality defines the requirements related
/// to the presence of the argument on the command line.
#[derive(Debug, Default, PartialEq, Eq, serde::Serialize)]
pub enum Optionality {
    /// Required indicates the argument is required
    /// exactly once.
    #[default]
    Required,
    /// Optional indicates the argument may or may not
    /// be present.
    Optional,
    /// Repeating indicates the argument may appear zero
    /// or more times.
    Repeating,
    /// Greedy is used for positional arguments which
    /// capture the all command line input up to the next flag or
    /// the end of the input.
    Greedy,
}

pub const INDENT: &str = "  ";
const DESCRIPTION_INDENT: usize = 20;
const WRAP_WIDTH: usize = 80;

/// Write command names and descriptions to an output string.
pub fn write_description(out: &mut String, cmd: &CommandInfo<'_>) {
    let mut current_line = INDENT.to_string();
    current_line.push_str(cmd.name);

    if cmd.description.is_empty() {
        new_line(&mut current_line, out);
        return;
    }

    if !indent_description(&mut current_line) {
        // Start the description on a new line if the flag names already
        // add up to more than DESCRIPTION_INDENT.
        new_line(&mut current_line, out);
    }

    let mut words = cmd.description.split(' ').peekable();
    while let Some(first_word) = words.next() {
        indent_description(&mut current_line);
        current_line.push_str(first_word);

        'inner: while let Some(&word) = words.peek() {
            if (char_len(&current_line) + char_len(word) + 1) > WRAP_WIDTH {
                new_line(&mut current_line, out);
                break 'inner;
            } else {
                // advance the iterator
                let _ = words.next();
                current_line.push(' ');
                current_line.push_str(word);
            }
        }
    }
    new_line(&mut current_line, out);
}

// Indent the current line in to DESCRIPTION_INDENT chars.
// Returns a boolean indicating whether or not spacing was added.
fn indent_description(line: &mut String) -> bool {
    let cur_len = char_len(line);
    if cur_len < DESCRIPTION_INDENT {
        let num_spaces = DESCRIPTION_INDENT - cur_len;
        line.extend(std::iter::repeat(' ').take(num_spaces));
        true
    } else {
        false
    }
}

fn char_len(s: &str) -> usize {
    s.chars().count()
}

// Append a newline and the current line to the output,
// clearing the current line.
fn new_line(current_line: &mut String, out: &mut String) {
    out.push('\n');
    out.push_str(current_line);
    current_line.truncate(0);
}
