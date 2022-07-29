// Copyright (c) 2022 Google LLC All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//! TODO

use super::{write_description, CommandInfo, INDENT};

#[cfg(feature = "serde")]
use serde_derive::{Deserialize, Serialize};

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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

    /// Returns a JSON encoded string of the usage information. This is intended to
    /// create a "machine readable" version of the help text to enable reference
    /// documentation generation.
    pub fn help_json_from_args(&self, command_name_arr: &[&str]) -> String {
        let command_name = command_name_arr.join(" ");

        let mut out = format!("{{\n\"name\": \"{}\"", command_name_arr.last().unwrap_or(&"<unknown>"));
        out.push_str(format!(",\n\"usage\": \"{}\"", self.usage_string(&command_name)).as_str());
        out.push_str(format!(",\n\"description\": \"{}\"", escape_json(&self.description.replace("{command_name}", &command_name))).as_str());

     
        out.push_str(",\n\"flags\": [");
        let mut first = true;
        for flag in self.flags {
            let short_flag = match flag.short {
                Some(ch) => format!("{}",ch),
                None => String::from("")
            };
            let arg_name = match flag.kind {
                HelpFieldKind::Option { arg_name } => arg_name,
                _ => ""
            };
            let optionality = match flag.optionality {
                HelpOptionality::Optional => "optional",
                HelpOptionality::Repeating => "repeating",
                HelpOptionality::None => "required"
            };

            if ! first {
                out.push_str(",\n");
            }
            first = false;
            out.push_str(format!("{{\"short\": \"{}\"",short_flag).as_str());
            out.push_str(format!(", \"long\": \"{}\"",flag.long).as_str());
            out.push_str(format!(", \"description\": \"{}\"", escape_json(flag.description)).as_str());
            out.push_str(format!(", \"arg_name\": \"{}\"",arg_name).as_str());
            out.push_str(format!(", \"optionality\": \"{}\"}}",optionality).as_str());

        }
        //Help flag is implicit
        if ! first {
            out.push_str(",\n");
        }
        out.push_str("{\"short\": \"\"");
        out.push_str(", \"long\": \"--help\"");
        out.push_str(format!(", \"description\": \"{}\"",HELP_FLAG.description).as_str());
        out.push_str(", \"arg_name\": \"\"");
        out.push_str(", \"optionality\": \"optional\"}");

        out.push(']');

        out.push_str(",\n\"positional\": [");
        first = true;
        for positional in self.positionals {
            let optionality = match positional.optionality {
                HelpOptionality::Optional => "optional",
                HelpOptionality::Repeating => "repeating",
                HelpOptionality::None => "required"
            };
            if ! first {
                out.push_str(",\n");
            }
            first = false;
            out.push_str(format!("{{\"name\": \"{}\"", positional.name).as_str());
            out.push_str(format!(", \"description\": \"{}\"", escape_json(positional.description)).as_str());
            out.push_str(format!(", \"optionality\": \"{}\"",optionality).as_str());
            out.push('}');
        }
        out.push(']');
        
        out.push_str(format!(",\n\"examples\": \"{}\"", escape_json(&self.examples.join("\n").replace("{command_name}", &command_name))).as_str());
        out.push_str(format!(",\n\"notes\": \"{}\"", escape_json(&self.notes.join("\n").replace("{command_name}", &command_name))).as_str());

        out.push_str(",\n\"error_codes\": [");
        first = true;
        for (code, description) in self.error_codes {
            if ! first {
                out.push_str(",\n");
            }
            first = false;
           out.push_str(format!("{{\"name\": \"{}\", \"description\": \"{}\"}}",code, escape_json(description)).as_str());
        }
        out.push(']');

        if let Some(subcommands) = self.subcommand {
            first = true;

        out.push_str(",\n\"subcommands\": [");
            for sub in subcommands.commands {
                if ! first {
                    out.push_str(",\n");
                }
                first = false;
                let sub_command_name =  [command_name_arr, &[sub.name]].concat();
                out.push_str(sub.info.help_json_from_args(&sub_command_name).as_str());
            }
            out.push(']');
        } else {
            out.push_str(",\n\"subcommands\": []");
        }

        out.push_str("\n}\n");
        out
    }


    /*
  left: `"{\n\"name\": \"test_arg_0\",\n\"usage\": \"test_arg_0 [--verbose] <command> [<args>]\",\n\"description\": \"Top level command with \\\"subcommands\\\".\",\n\"flags\": [{\"short\": \"\", \"long\": \"--verbose\", \"description\": \"show verbose output\", \"arg_name\": \"\", \"optionality\": \"optional\"},\n{\"short\": \"\", \"long\": \"--help\", \"description\": \"display usage information\", \"arg_name\": \"\", \"optionality\": \"optional\"}],\n\"positional\": [],\n\"examples\": \"\",\n\"notes\": \"\",\n\"error_codes\": [],\n\"subcommands\": [{\n\"name\": \"one\",\n\"usage\": \"test_arg_0 one <root> <trunk> [<leaves>]\",\n\"description\": \"Command1 args are used for Command1.\",\n\"flags\": [{\"short\": \"\", \"long\": \"--help\", \"description\": \"display usage information\", \"arg_name\": \"\", \"optionality\": \"optional\"}],\n\"positional\": [{\"name\": \"root\", \"description\": \"the \\\"root\\\" position.\", \"optionality\": \"required\"},\n{\"name\": \"trunk\", \"description\": \"trunk value\", \"optionality\": \"required\"},\n{\"name\": \"leaves\", \"description\": \"leaves. There can be zero leaves, defaults to hello.\", \"optionality\": \"optional\"}],\n\"examples\": \"\",\n\"notes\": \"\",\n\"error_codes\": [],\n\"subcommands\": []\n}\n,\n{\n\"name\": \"two\",\n\"usage\": \"test_arg_0 two [--power] --required <required> [-s <speed>] [--link <url...>]\",\n\"description\": \"Command2 args are used for Command2.\",\n\"flags\": [{\"short\": \"\", \"long\": \"--power\", \"description\": \"should the power be on. \\\"Quoted value\\\" should work too.\", \"arg_name\": \"\", \"optionality\": \"optional\"},\n{\"short\": \"\", \"long\": \"--required\", \"description\": \"option that is required because of no default and not Option<>.\", \"arg_name\": \"required_flag\", \"optionality\": \"required\"},\n{\"short\": \"s\", \"long\": \"--speed\", \"description\": \"optional speed if not specified it is None.\", \"arg_name\": \"speed\", \"optionality\": \"optional\"},\n{\"short\": \"\", \"long\": \"--link\", \"description\": \"repeatable option.\", \"arg_name\": \"url\", \"optionality\": \"repeating\"},\n{\"short\": \"\", \"long\": \"--help\", \"description\": \"display usage information\", \"arg_name\": \"\", \"optionality\": \"optional\"}],\n\"positional\": [],\n\"examples\": \"\",\n\"notes\": \"\",\n\"error_codes\": [],\n\"subcommands\": []\n}\n,\n{\n\"name\": \"three\",\n\"usage\": \"test_arg_0 three\",\n\"description\": \"Command3 args are used for Command3 which has no options or arguments.\",\n\"flags\": [{\"short\": \"\", \"long\": \"--help\", \"description\": \"display usage information\", \"arg_name\": \"\", \"optionality\": \"optional\"}],\n\"positional\": [],\n\"examples\": \"\",\n\"notes\": \"\",\n\"error_codes\": [],\n\"subcommands\": []\n}\n]\n}\n"`,
 right: `"{\n\"name\": \"test_arg_0\",\n\"usage\": \"test_arg_0 [--verbose] <command> [<args>]\",\n\"description\": \"Top level command with \\\"subcommands\\\".\",\n\"flags\": [{\"short\": \"\", \"long\": \"--verbose\", \"description\": \"show verbose output\", \"arg_name\": \"\", \"optionality\": \"optional\"},\n{\"short\": \"\", \"long\": \"--help\", \"description\": \"display usage information\", \"arg_name\": \"\", \"optionality\": \"optional\"}],\n\"positional\": [],\n\"examples\": \"\",\n\"notes\": \"\",\n\"error_codes\": [],\n\"subcommands\": [{\n\"name\": \"one\",\n\"usage\": \"test_arg_0 one <root> <trunk> [<leaves>]\",\n\"description\": \"Command1 args are used for Command1.\",\n\"flags\": [{\"short\": \"\", \"long\": \"--help\", \"description\": \"display usage information\", \"arg_name\": \"\", \"optionality\": \"optional\"}],\n\"positional\": [{\"name\": \"root\", \"description\": \"the \\\"root\\\" position.\", \"optionality\": \"required\"},\n{\"name\": \"trunk\", \"description\": \"trunk value\", \"optionality\": \"required\"},\n{\"name\": \"leaves\", \"description\": \"leaves. There can be zero leaves, defaults to hello.\", \"optionality\": \"optional\"}],\n\"examples\": \"\",\n\"notes\": \"\",\n\"error_codes\": [],\n\"subcommands\": []\n}\n,\n{\n\"name\": \"two\",\n\"usage\": \"test_arg_0 two [--power] --required <required> [-s <speed>] [--link <url...>]\",\n\"description\": \"Command2 args are used for Command2.\",\n\"flags\": [{\"short\": \"\", \"long\": \"--power\", \"description\": \"should the power be on. \\\"Quoted value\\\" should work too.\", \"arg_name\": \"\", \"optionality\": \"optional\"},\n{\"short\": \"\", \"long\": \"--required\", \"description\": \"option that is required because of no default and not Option<>.\", \"arg_name\": \"required\", \"optionality\": \"required\"},\n{\"short\": \"s\", \"long\": \"--speed\", \"description\": \"optional speed if not specified it is None.\", \"arg_name\": \"speed\", \"optionality\": \"optional\"},\n{\"short\": \"\", \"long\": \"--link\", \"description\": \"repeatable option.\", \"arg_name\": \"url\", \"optionality\": \"repeating\"},\n{\"short\": \"\", \"long\": \"--help\", \"description\": \"display usage information\", \"arg_name\": \"\", \"optionality\": \"optional\"}],\n\"positional\": [],\n\"examples\": \"\",\n\"notes\": \"\",\n\"error_codes\": [],\n\"subcommands\": []\n}\n,\n{\n\"name\": \"three\",\n\"usage\": \"test_arg_0 three\",\n\"description\": \"Command3 args are used for Command3 which has no options or arguments.\",\n\"flags\": [{\"short\": \"\", \"long\": \"--help\", \"description\": \"display usage information\", \"arg_name\": \"\", \"optionality\": \"optional\"}],\n\"positional\": [],\n\"examples\": \"\",\n\"notes\": \"\",\n\"error_codes\": [],\n\"subcommands\": []\n}\n]\n}\n"`',

 */

    fn usage_string(&self, command_name: &String) -> String {
        let mut out =  command_name.to_string();

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

        out
    }
}

fn escape_json(value: &str) -> String {
    value.replace('\n', r#"\n"#).replace('"', r#"\""#)
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HelpSubCommandsInfo<'a> {
    /// TODO
    pub optional: bool,
    /// TODO
    pub commands: &'a [&'a HelpSubCommandInfo<'a>],
}

/// TODO
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HelpSubCommandInfo<'a> {
    /// TODO
    pub name: &'a str,
    /// TODO
    pub info: &'a HelpInfo<'a>,
}

/// TODO
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HelpPositionalInfo<'a> {
    /// TODO
    pub name: &'a str,
    /// TODO
    pub description: &'a str,
    /// TODO
    pub optionality: HelpOptionality
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
