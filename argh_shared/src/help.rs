// Copyright (c) 2022 Google LLC All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//! TODO

use super::{write_description, CommandInfo, INDENT};

const SECTION_SEPARATOR: &str = "\n\n";

const HELP_FLAG: HelpFlagInfo = HelpFlagInfo {
    short: None,
    long: "--help",
    description: "display usage information",
    optionality: HelpOptionality::Optional,
    kind: HelpFieldKind::Switch,
};

/// TODO
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HelpInfo<'a> {
    /// TODO
    pub description: &'a str,
    /// TODO
    pub examples: &'a [&'a str],
    /// TODO
    pub notes: &'a [&'a str],
    /// TODO
    pub flags: &'a [HelpFlagInfo<'a>],
    /// TODO
    pub positionals: &'a [HelpPositionalInfo<'a>],
    /// TODO
    pub subcommand: Option<&'a HelpSubCommandsInfo<'a>>,
    /// TODO
    pub error_codes: &'a [(isize, &'a str)],
}

fn help_section(out: &mut String, command_name: &str, heading: &str, sections: &[&str]) {
    if !sections.is_empty() {
        out.push_str(SECTION_SEPARATOR);
        for section in sections {
            let section = section.replace("{command_name}", command_name);

            out.push_str(heading);
            for line in section.split('\n') {
                out.push('\n');
                out.push_str(INDENT);
                out.push_str(line);
            }
        }
    }
}

impl<'a> HelpInfo<'a> {
    /// TODO
    pub fn help(&self, command_name: &[&str]) -> String {
        let command_name = command_name.join(" ");
        let mut out = format!("Usage: {}", command_name);

        for positional in self.positionals {
            out.push(' ');
            positional.help_usage(&mut out);
        }

        for flag in self.flags {
            out.push(' ');
            flag.help_usage(&mut out);
        }

        if let Some(subcommand) = &self.subcommand {
            out.push(' ');
            if subcommand.optional {
                out.push('[');
            }
            out.push_str("<command>");
            if subcommand.optional {
                out.push(']');
            }
            out.push_str(" [<args>]");
        }

        out.push_str(SECTION_SEPARATOR);

        out.push_str(self.description);

        if !self.positionals.is_empty() {
            out.push_str(SECTION_SEPARATOR);
            out.push_str("Positional Arguments:");
            for positional in self.positionals {
                positional.help_description(&mut out);
            }
        }

        out.push_str(SECTION_SEPARATOR);
        out.push_str("Options:");
        for flag in self.flags {
            flag.help_description(&mut out);
        }

        // Also include "help"
        HELP_FLAG.help_description(&mut out);

        if let Some(subcommand) = &self.subcommand {
            out.push_str(SECTION_SEPARATOR);
            out.push_str("Commands:");
            for cmd in subcommand.commands {
                let info = CommandInfo { name: cmd.name, description: cmd.info.description };
                write_description(&mut out, &info);
            }
        }

        help_section(&mut out, &command_name, "Examples:", self.examples);

        help_section(&mut out, &command_name, "Notes:", self.notes);

        if !self.error_codes.is_empty() {
            out.push_str(SECTION_SEPARATOR);
            out.push_str("Error codes:");
            write_error_codes(&mut out, self.error_codes);
        }

        out.push('\n');

        out
    }
}

fn write_error_codes(out: &mut String, error_codes: &[(isize, &str)]) {
    for (code, text) in error_codes {
        out.push('\n');
        out.push_str(INDENT);
        out.push_str(&format!("{} {}", code, text));
    }
}

/// TODO
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpSubCommandsInfo<'a> {
    /// TODO
    pub optional: bool,
    /// TODO
    pub commands: &'a [&'a HelpSubCommandInfo<'a>],
}

/// TODO
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpSubCommandInfo<'a> {
    /// TODO
    pub name: &'a str,
    /// TODO
    pub info: &'a HelpInfo<'a>,
}

/// TODO
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HelpOptionality {
    /// TODO
    None,
    /// TODO
    Optional,
    /// TODO
    Repeating,
}

impl HelpOptionality {
    /// TODO
    fn is_required(&self) -> bool {
        matches!(self, HelpOptionality::None)
    }
}

/// TODO
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpPositionalInfo<'a> {
    /// TODO
    pub name: &'a str,
    /// TODO
    pub description: &'a str,
    /// TODO
    pub optionality: HelpOptionality,
}

impl<'a> HelpPositionalInfo<'a> {
    /// Add positional arguments like `[<foo>...]` to a help format string.
    pub fn help_usage(&self, out: &mut String) {
        if !self.optionality.is_required() {
            out.push('[');
        }

        out.push('<');
        out.push_str(self.name);

        if self.optionality == HelpOptionality::Repeating {
            out.push_str("...");
        }

        out.push('>');

        if !self.optionality.is_required() {
            out.push(']');
        }
    }

    /// Describes a positional argument like this:
    ///  hello       positional argument description
    pub fn help_description(&self, out: &mut String) {
        let info = CommandInfo { name: self.name, description: self.description };
        write_description(out, &info);
    }
}

/// TODO
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpFlagInfo<'a> {
    /// TODO
    pub short: Option<char>,
    /// TODO
    pub long: &'a str,
    /// TODO
    pub description: &'a str,
    /// TODO
    pub optionality: HelpOptionality,
    /// TODO
    pub kind: HelpFieldKind<'a>,
}

/// TODO
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelpFieldKind<'a> {
    /// TODO
    Switch,
    /// TODO
    Option {
        /// TODO
        arg_name: &'a str,
    },
}

impl<'a> HelpFlagInfo<'a> {
    /// Add options like `[-f <foo>]` to a help format string.
    /// This function must only be called on options (things with `long_name.is_some()`)
    pub fn help_usage(&self, out: &mut String) {
        if !self.optionality.is_required() {
            out.push('[');
        }

        if let Some(short) = self.short {
            out.push('-');
            out.push(short);
        } else {
            out.push_str(self.long);
        }

        match self.kind {
            HelpFieldKind::Switch => {}
            HelpFieldKind::Option { arg_name } => {
                out.push_str(" <");
                out.push_str(arg_name);

                if self.optionality == HelpOptionality::Repeating {
                    out.push_str("...");
                }

                out.push('>');
            }
        }

        if !self.optionality.is_required() {
            out.push(']');
        }
    }

    /// Describes an option like this:
    ///  -f, --force       force, ignore minor errors. This description
    ///                    is so long that it wraps to the next line.
    pub fn help_description(&self, out: &mut String) {
        let mut name = String::new();
        if let Some(short) = self.short {
            name.push('-');
            name.push(short);
            name.push_str(", ");
        }
        name.push_str(self.long);

        let info = CommandInfo { name: &name, description: self.description };
        write_description(out, &info);
    }
}
