// Copyright (c) 2026 Google LLC All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//! Generation of completions for Zsh.

use crate::Generator;
use argh_shared::{CommandInfoWithArgs, FlagInfoKind};
use std::fmt::Write;

/// A generator for Zsh shell completions.
pub struct Zsh;

impl Generator for Zsh {
    fn generate(cmd_name: &str, cmd: &CommandInfoWithArgs<'_>) -> String {
        let mut out = String::new();

        writeln!(&mut out, "#compdef {}", cmd_name).unwrap();
        writeln!(&mut out).unwrap();
        writeln!(&mut out, "_{}() {{", cmd_name).unwrap();
        writeln!(&mut out, "    local context state state_descr line").unwrap();
        writeln!(&mut out, "    typeset -A opt_args").unwrap();
        writeln!(&mut out).unwrap();

        generate_zsh_args(&mut out, cmd_name, cmd, 1);

        writeln!(&mut out, "}}").unwrap();
        writeln!(&mut out).unwrap();

        // Generate functions for subcommands
        for subcmd in &cmd.commands {
            generate_zsh_subcmd(&mut out, cmd_name, &subcmd.command);
        }

        writeln!(&mut out, "if [[ $funcstack[1] == _{} ]]; then", cmd_name).unwrap();
        writeln!(&mut out, "    _{} \"$@\"", cmd_name).unwrap();
        writeln!(&mut out, "else").unwrap();
        writeln!(&mut out, "    compdef _{} {}", cmd_name, cmd_name).unwrap();
        writeln!(&mut out, "fi").unwrap();

        out
    }
}

fn generate_zsh_subcmd(out: &mut String, prefix: &str, cmd: &CommandInfoWithArgs<'_>) {
    let full_name = format!("{}_{}", prefix, cmd.name);
    writeln!(out, "_{}() {{", full_name).unwrap();
    writeln!(out, "    local context state state_descr line").unwrap();
    writeln!(out, "    typeset -A opt_args").unwrap();
    writeln!(out).unwrap();

    generate_zsh_args(out, &full_name, cmd, 1);

    writeln!(out, "}}").unwrap();
    writeln!(out).unwrap();

    for subcmd in &cmd.commands {
        generate_zsh_subcmd(out, &full_name, &subcmd.command);
    }
}

fn generate_zsh_args(out: &mut String, prefix: &str, cmd: &CommandInfoWithArgs<'_>, indent: usize) {
    let ind = "    ".repeat(indent);
    writeln!(out, "{}_arguments -s -S \\", ind).unwrap();

    for flag in cmd.flags {
        let mut def = String::new();
        let desc = flag
            .description
            .replace("[", "\\[")
            .replace("]", "\\]")
            .replace("'", "'\\''")
            .replace(":", "\\:");

        let has_short = flag.short.is_some();
        let has_long = !flag.long.is_empty();

        if has_short && has_long {
            let short = format!("-{}", flag.short.unwrap());
            def.push_str(&format!(
                "'({} {})'{{{},{}}}[{}]",
                short, flag.long, short, flag.long, desc
            ));
        } else if has_long {
            def.push_str(&format!("'{}[{}]'", flag.long, desc));
        } else if has_short {
            let short = format!("-{}", flag.short.unwrap());
            def.push_str(&format!("'{}[{}]'", short, desc));
        }

        if let FlagInfoKind::Option { .. } = flag.kind {
            def.push_str("': :'"); // generic argument
        }

        writeln!(out, "{}    {} \\", ind, def).unwrap();
    }

    if !cmd.commands.is_empty() {
        writeln!(out, "{}    '*::command:->subcmd' && return 0", ind).unwrap();
        writeln!(out).unwrap();
        writeln!(out, "{}case $state in", ind).unwrap();
        writeln!(out, "{}    (subcmd)", ind).unwrap();
        writeln!(out, "{}        local -a subcommands", ind).unwrap();
        writeln!(out, "{}        subcommands=(", ind).unwrap();
        for subcmd in &cmd.commands {
            let desc = subcmd.command.description.replace("'", "'\\''").replace(":", "\\:");
            writeln!(out, "{}            '{}:{}'", ind, subcmd.name, desc).unwrap();
        }
        writeln!(out, "{}        )", ind).unwrap();
        writeln!(out, "{}        _describe -t commands '{} commands' subcommands", ind, cmd.name)
            .unwrap();
        writeln!(out, "{}        if (( CURRENT == 1 )); then", ind).unwrap();
        writeln!(out, "{}            return", ind).unwrap();
        writeln!(out, "{}        fi", ind).unwrap();
        writeln!(out, "{}        local cmd=$words[1]", ind).unwrap();
        writeln!(out, "{}        curcontext=\"${{curcontext%:*:*}}:{}-$cmd\"", ind, prefix)
            .unwrap();
        writeln!(out, "{}        case $cmd in", ind).unwrap();
        for subcmd in &cmd.commands {
            writeln!(out, "{}            ({})", ind, subcmd.name).unwrap();
            writeln!(out, "{}                _{}_{}", ind, prefix, subcmd.name).unwrap();
            writeln!(out, "{}                ;;", ind).unwrap();
        }
        writeln!(out, "{}        esac", ind).unwrap();
        writeln!(out, "{}        ;;", ind).unwrap();
        writeln!(out, "{}esac", ind).unwrap();
    } else {
        // Just cap it off if no subcommands
        writeln!(out, "{}    && return 0", ind).unwrap();
    }
}
