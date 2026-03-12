// Copyright (c) 2026 Google LLC All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//! Generation of completions for Nushell.

use crate::Generator;
use argh_shared::{CommandInfoWithArgs, FlagInfoKind, Optionality};
use std::fmt::Write;

/// A generator for Nushell shell completions.
pub struct Nushell;

impl Generator for Nushell {
    fn generate(cmd_name: &str, cmd: &CommandInfoWithArgs<'_>) -> String {
        let mut out = String::new();
        generate_nushell_cmd(&mut out, cmd_name, cmd);
        out
    }
}

fn generate_nushell_cmd(out: &mut String, cmd_name: &str, cmd: &CommandInfoWithArgs<'_>) {
    // Generate the extern block for the current command
    writeln!(out, "export extern \"{}\" [", cmd_name).unwrap();

    // Generate flags
    for flag in cmd.flags {
        let mut flag_def = String::new();

        if !flag.long.is_empty() {
            flag_def.push_str(flag.long);
        }

        if let Some(short) = flag.short {
            if flag.long.is_empty() {
                // If there is only a short flag, we must output it as a short option.
                // Nushell requires some long name for just `-s`, but standard is `-s`.
                // If it's only short, typically `-$short`
                flag_def.push_str(&format!("-{}", short));
            } else {
                flag_def.push_str(&format!("(-{})", short));
            }
        }

        if let FlagInfoKind::Option { .. } = flag.kind {
            flag_def.push_str(": string");
        }

        if !flag.description.is_empty() {
            flag_def.push_str(&format!(" # {}", flag.description));
        }

        writeln!(out, "    {}", flag_def).unwrap();
    }

    // Generate positional arguments
    for pos in cmd.positionals {
        let name = if pos.name.is_empty() { "arg" } else { pos.name };

        let mut pos_def = String::new();
        match pos.optionality {
            Optionality::Required => pos_def.push_str(&format!("{}: string", name)),
            Optionality::Optional => pos_def.push_str(&format!("{}?: string", name)),
            Optionality::Repeating | Optionality::Greedy => {
                pos_def.push_str(&format!("...{}: string", name))
            }
        }

        if !pos.description.is_empty() {
            pos_def.push_str(&format!(" # {}", pos.description));
        }

        writeln!(out, "    {}", pos_def).unwrap();
    }

    writeln!(out, "]").unwrap();
    writeln!(out).unwrap();

    // Recurse for subcommands
    for subcmd in &cmd.commands {
        let next_cmd_name = format!("{} {}", cmd_name, subcmd.name);
        generate_nushell_cmd(out, &next_cmd_name, &subcmd.command);
    }
}
