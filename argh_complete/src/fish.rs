// Copyright (c) 2026 Google LLC All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//! Generation of completions for Fish.

use crate::Generator;
use argh_shared::{CommandInfoWithArgs, FlagInfoKind};
use std::fmt::Write;

/// A generator for Fish shell completions.
pub struct Fish;

fn collect_value_flags(cmd: &CommandInfoWithArgs<'_>, out: &mut Vec<String>) {
    for flag in cmd.flags {
        if let FlagInfoKind::Option { .. } = flag.kind {
            if let Some(short) = flag.short {
                out.push(short.to_string());
            }
            let long = flag.long.trim_start_matches('-');
            if !long.is_empty() {
                out.push(long.to_string());
            }
        }
    }
    for sub in &cmd.commands {
        collect_value_flags(&sub.command, out);
    }
}

impl Generator for Fish {
    fn generate(cmd_name: &str, cmd: &CommandInfoWithArgs<'_>) -> String {
        let mut out = String::new();

        let mut value_flags = Vec::new();
        collect_value_flags(cmd, &mut value_flags);

        value_flags.sort();
        value_flags.dedup();

        let mut known_value_flags_str = String::new();
        for flag in value_flags {
            known_value_flags_str.push_str(&format!("'{flag}' "));
        }

        // Generate the custom parsing state machine function that replaces `__fish_seen_subcommand_from`
        writeln!(
            out,
            "function __fish_{cmd_name}_using_command\n\
            \x20   set -l cmd (commandline -xpc)\n\
            \x20   set -e cmd[1]\n\
            \x20   set -l target_cmd $argv\n\
            \n\
            \x20   set -l known_value_flags {known_value_flags_str}\n\
            \n\
            \x20   set -l i 1\n\
            \x20   set -l cleaned_cmd\n\
            \x20   while test $i -le (count $cmd)\n\
            \x20       set -l arg $cmd[$i]\n\
            \x20       if string match -q -- \"-*\" $arg\n\
            \x20           set -l pure_flag (string replace -r '^--?' '' -- $arg)\n\
            \x20           if contains -- $pure_flag $known_value_flags\n\
            \x20               set -l next_i (math $i + 1)\n\
            \x20               if test $next_i -le (count $cmd)\n\
            \x20                   if not string match -q -- \"-*\" $cmd[$next_i]\n\
            \x20                       set i $next_i\n\
            \x20                   end\n\
            \x20               end\n\
            \x20           end\n\
            \x20       else\n\
            \x20           set -a cleaned_cmd $arg\n\
            \x20       end\n\
            \x20       set i (math $i + 1)\n\
            \x20   end\n\
            \n\
            \x20   set -l count_cleaned (count $cleaned_cmd)\n\
            \x20   set -l count_target (count $target_cmd)\n\
            \x20   \n\
            \x20   if test $count_cleaned -ne $count_target\n\
            \x20       return 1\n\
            \x20   end\n\
            \x20   \n\
            \x20   for i in (seq $count_target)\n\
            \x20       if test $cleaned_cmd[$i] != $target_cmd[$i]\n\
            \x20           return 1\n\
            \x20       end\n\
            \x20   end\n\
            \x20   \n\
            \x20   return 0\n\
            end\n"
        )
        .unwrap();

        generate_fish_cmd(&mut out, cmd_name, cmd_name, cmd, &[]);
        out
    }
}

fn generate_fish_cmd(
    out: &mut String,
    bin_name: &str,
    base_cmd: &str,
    cmd: &CommandInfoWithArgs<'_>,
    parent_subcommands: &[&str],
) {
    let joined_condition = if parent_subcommands.is_empty() {
        format!("-n '__fish_{bin_name}_using_command'")
    } else {
        format!("-n '__fish_{bin_name}_using_command {}'", parent_subcommands.join(" "))
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
        generate_fish_cmd(out, bin_name, base_cmd, &subcmd.command, &new_parents);
    }
}
