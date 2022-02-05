// Copyright (c) 2022 Google LLC All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//! TODO

use {
    argh_shared::{write_description, CommandInfo, INDENT},
    std::fmt,
};

const SECTION_SEPARATOR: &str = "\n\n";

const HELP_FLAG: HelpFlagInfo = HelpFlagInfo {
    short: None,
    long: "--help",
    description: "display usage information",
    optionality: HelpOptionality::Optional,
    kind: HelpFieldKind::Switch,
};

/// TODO
pub trait Help {
    /// TODO
    const HELP_INFO: &'static HelpInfo;
}

/// TODO
pub trait HelpSubCommands {
    /// TODO
    const HELP_INFO: &'static HelpSubCommandsInfo;
}

/// TODO
pub trait HelpSubCommand {
    /// TODO
    const HELP_INFO: &'static HelpSubCommandInfo;
}

impl<T: HelpSubCommand> HelpSubCommands for T {
    /// TODO
    const HELP_INFO: &'static HelpSubCommandsInfo =
        &HelpSubCommandsInfo { optional: false, commands: &[<T as HelpSubCommand>::HELP_INFO] };
}

/// TODO
pub struct HelpInfo {
    /// TODO
    pub description: &'static str,
    /// TODO
    pub examples: &'static [fn(&[&str]) -> String],
    /// TODO
    pub notes: &'static [fn(&[&str]) -> String],
    /// TODO
    pub flags: &'static [&'static HelpFlagInfo],
    /// TODO
    pub positionals: &'static [&'static HelpPositionalInfo],
    /// TODO
    pub subcommand: Option<&'static HelpSubCommandsInfo>,
    /// TODO
    pub error_codes: &'static [(isize, &'static str)],
}

fn help_section(
    out: &mut String,
    command_name: &[&str],
    heading: &str,
    sections: &[fn(&[&str]) -> String],
) {
    if !sections.is_empty() {
        out.push_str(SECTION_SEPARATOR);
        for section_fn in sections {
            let section = section_fn(command_name);

            out.push_str(heading);
            for line in section.split('\n') {
                out.push('\n');
                out.push_str(INDENT);
                out.push_str(line);
            }
        }
    }
}

impl HelpInfo {
    /// TODO
    pub fn help(&self, command_name: &[&str]) -> String {
        let mut out = format!("Usage: {}", command_name.join(" "));

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

        help_section(&mut out, command_name, "Examples:", self.examples);

        help_section(&mut out, command_name, "Notes:", self.notes);

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

impl fmt::Debug for HelpInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let examples = self.examples.iter().map(|f| f(&["{command_name}"])).collect::<Vec<_>>();
        let notes = self.notes.iter().map(|f| f(&["{command_name}"])).collect::<Vec<_>>();
        f.debug_struct("HelpInfo")
            .field("description", &self.description)
            .field("examples", &examples)
            .field("notes", &notes)
            .field("flags", &self.flags)
            .field("positionals", &self.positionals)
            .field("subcommand", &self.subcommand)
            .field("error_codes", &self.error_codes)
            .finish()
    }
}

/// TODO
#[derive(Debug)]
pub struct HelpSubCommandsInfo {
    /// TODO
    pub optional: bool,
    /// TODO
    pub commands: &'static [&'static HelpSubCommandInfo],
}

/// TODO
#[derive(Debug)]
pub struct HelpSubCommandInfo {
    /// TODO
    pub name: &'static str,
    /// TODO
    pub info: &'static HelpInfo,
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
#[derive(Debug)]
pub struct HelpPositionalInfo {
    /// TODO
    pub name: &'static str,
    /// TODO
    pub description: &'static str,
    /// TODO
    pub optionality: HelpOptionality,
}

impl HelpPositionalInfo {
    /// TODO
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

    /// TODO
    pub fn help_description(&self, out: &mut String) {
        let info = CommandInfo { name: self.name, description: self.description };
        write_description(out, &info);
    }
}

/// TODO
#[derive(Debug)]
pub struct HelpFlagInfo {
    /// TODO
    pub short: Option<char>,
    /// TODO
    pub long: &'static str,
    /// TODO
    pub description: &'static str,
    /// TODO
    pub optionality: HelpOptionality,
    /// TODO
    pub kind: HelpFieldKind,
}

/// TODO
#[derive(Debug)]
pub enum HelpFieldKind {
    /// TODO
    Switch,
    /// TODO
    Option {
        /// TODO
        arg_name: &'static str,
    },
}

impl HelpFlagInfo {
    /// TODO
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

    /// TODO
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
