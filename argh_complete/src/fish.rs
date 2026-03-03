// Copyright (c) 2026 Google LLC All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//! Generation of completions for Fish.

use crate::Generator;
use argh_shared::{CommandInfoWithArgs, FlagInfoKind};
use std::fmt::Write;

/// A generator for Fish shell completions.
pub struct Fish;

impl Generator for Fish {
    fn generate(cmd_name: &str, cmd: &CommandInfoWithArgs<'_>) -> String {
        let mut out = String::new();

        // In Fish, `complete -c cmd_name ...` is used to register completions.
        // We can do this recursively for subcommands, but we need to track the parent command tree.
        // For simplicity, we'll start with a flat generation for the root command and its immediate children.

        generate_fish_cmd(&mut out, cmd_name, cmd, "");

        for subcmd in &cmd.commands {
            // Fish subcommand completions usually require checking if the subcommand is the current token
            // using `-n '__fish_seen_subcommand_from <subcmd>'` or similar.
            let subcmd_condition = format!("-n \"__fish_seen_subcommand_from {}\"", subcmd.name);
            generate_fish_cmd(&mut out, cmd_name, &subcmd.command, &subcmd_condition);
        }

        out
    }
}

fn generate_fish_cmd(
    out: &mut String,
    base_cmd: &str,
    cmd: &CommandInfoWithArgs<'_>,
    condition: &str,
) {
    for flag in cmd.flags {
        let mut line = format!("complete -c {}", base_cmd);
        if !condition.is_empty() {
            line.push(' ');
            line.push_str(condition);
        }

        // Use long syntax:
        if !flag.long.is_empty() {
            let stripped_long = flag.long.trim_start_matches('-');
            if !stripped_long.is_empty() {
                line.push_str(&format!(" -l {}", stripped_long));
            }
        }

        if let Some(short) = flag.short {
            line.push_str(&format!(" -s {}", short));
        }

        if let FlagInfoKind::Option { arg_name } = flag.kind {
            // Options usually take arguments, so we add `-r` (requires argument) and possibly `-d` (description)
            line.push_str(" -r");
            let _ = arg_name; // Maybe use argument name in description
        }

        if !flag.description.is_empty() {
            // escape single quotes in description
            let description = flag.description.replace("'", "\\'");
            line.push_str(&format!(" -d '{}'", description));
        }

        writeln!(out, "{}", line).unwrap();
    }

    // Subcommands themselves are completions.
    for subcmd in &cmd.commands {
        let mut line = format!("complete -c {}", base_cmd);
        // Note: Fish typically handles subcommands by defining them as arguments that can be taken without a flag
        // `-f` means no file completion, `-a` means an argument.
        if !condition.is_empty() {
            line.push_str(&format!(" {} -f -a '{}'", condition, subcmd.name));
        } else {
            // Check if we are at the top level, we might want to ensure no subcommand has been seen yet.
            line.push_str(&format!(
                " -n \"not __fish_seen_subcommand_from ...\" -f -a '{}'",
                subcmd.name
            ));
            // A more robust way in Fish is custom conditions, but here's a simpler one:
            // Just complete the subcommand if it's not starting with a dash.
            // Often we define a fish function `__fish_cmd_needs_command` and use it.
            // For now, let's just register it as a top level argument `-a subcmd.name`
            line = format!(
                "complete -c {} -f -a '{}' -d '{}'",
                base_cmd,
                subcmd.name,
                subcmd.command.description.replace("'", "\\'")
            );
        }
        writeln!(out, "{}", line).unwrap();
    }
}
