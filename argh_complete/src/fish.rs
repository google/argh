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
        generate_fish_cmd(&mut out, cmd_name, cmd, &[]);
        out
    }
}

fn generate_fish_cmd(
    out: &mut String,
    base_cmd: &str,
    cmd: &CommandInfoWithArgs<'_>,
    parent_subcommands: &[&str],
) {
    // Condition for the current command's flags and immediate subcommands.
    // It must be active (all parents seen) AND no children seen yet.
    let mut conditions = Vec::new();

    // 1. Requirement: All parent subcommands must be effectively "seen" (in order).
    // actually `__fish_seen_subcommand_from` handles the check if *any* of them are seen,
    // but for specific nesting, we usually want to say "we have seen parent X" and "we have NOT seen any child of X yet".

    // For the root command, we don't need `__fish_seen_subcommand_from`.
    // For subcommand `A`, we need `__fish_seen_subcommand_from A`.
    // For nested `A B`, we need `__fish_seen_subcommand_from B`.
    // The issue with `__fish_seen_subcommand_from` is it returns true if the subcommand is present *anywhere*.
    // However, typical fish completion scripts use this pattern:
    // `complete -c cmd -n '__fish_seen_subcommand_from sub' ...`
    // This implies we are "in" the subcommand.

    // If we have parents, the last parent must be seen.
    if let Some(last_parent) = parent_subcommands.last() {
        conditions.push(format!("__fish_seen_subcommand_from {}", last_parent));
    } else {
        // Root command: ensure NO subcommands are seen (if we want to prevent root flags from showing up in subcommands).
        // BUT strict `not __fish_seen_subcommand_from ...` for ALL descendants is hard.
        // Usually, we just check immediate children to distinguish "root context" from "subcommand context".
        conditions.push("not __fish_seen_subcommand_from".to_string());
        for sub in &cmd.commands {
            conditions[0].push_str(&format!(" {}", sub.name));
        }
    }

    // 2. Requirement: No immediate child of THIS command must be seen.
    if !cmd.commands.is_empty() && !parent_subcommands.is_empty() {
        let mut not_seen_child = String::from("not __fish_seen_subcommand_from");
        for sub in &cmd.commands {
            not_seen_child.push_str(&format!(" {}", sub.name));
        }
        conditions.push(not_seen_child);
    }

    let joined_condition = if conditions.is_empty() {
        String::new()
    } else {
        let parts: Vec<String> = conditions.iter().map(|c| format!("-n '{}'", c)).collect();
        parts.join(" ")
    };

    // If the command has no positional arguments, disable file completion.
    let no_files = if cmd.positionals.is_empty() { " -f" } else { "" };

    // Generate flags for this command
    for flag in cmd.flags {
        let mut line = format!("complete -c {}", base_cmd);
        if !joined_condition.is_empty() {
            line.push(' ');
            line.push_str(&joined_condition);
        }

        // Add -f if applicable
        line.push_str(no_files);

        if !flag.long.is_empty() {
            let stripped_long = flag.long.trim_start_matches('-');
            if !stripped_long.is_empty() {
                line.push_str(&format!(" -l {}", stripped_long));
            }
        }

        if let Some(short) = flag.short {
            line.push_str(&format!(" -s {}", short));
        }

        if let FlagInfoKind::Option { .. } = flag.kind {
            line.push_str(" -r");
        }

        if !flag.description.is_empty() {
            let description = flag.description.replace("'", "\\'");
            line.push_str(&format!(" -d '{}'", description));
        }

        writeln!(out, "{}", line).unwrap();
    }

    // Generate immediate subcommands (as arguments to this command)
    for subcmd in &cmd.commands {
        let mut line = format!("complete -c {}", base_cmd);
        if !joined_condition.is_empty() {
            line.push(' ');
            line.push_str(&joined_condition);
        }
        // Subcommands are just arguments that don't take files
        line.push_str(&format!(
            " -f -a '{}' -d '{}'",
            subcmd.name,
            subcmd.command.description.replace("'", "\\'")
        ));
        writeln!(out, "{}", line).unwrap();
    }

    // Recurse
    for subcmd in &cmd.commands {
        let mut new_parents = parent_subcommands.to_vec();
        new_parents.push(subcmd.name);
        generate_fish_cmd(out, base_cmd, &subcmd.command, &new_parents);
    }
}
