// Copyright (c) 2026 Google LLC All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

#[cfg(test)]
use crate::Generator;
#[cfg(test)]
use argh_shared::{CommandInfoWithArgs, FlagInfo, FlagInfoKind, Optionality, SubCommandInfo};

#[cfg(test)]
fn make_mock_command() -> CommandInfoWithArgs<'static> {
    let subcmd_info =
        CommandInfoWithArgs { name: "subcmd", description: "a sub command", ..Default::default() };

    CommandInfoWithArgs {
        name: "mycmd",
        description: "A standard command",
        flags: &[FlagInfo {
            kind: FlagInfoKind::Switch,
            optionality: Optionality::Optional,
            long: "--verbose",
            short: Some('v'),
            description: "verbose output",
            hidden: false,
        }],
        commands: vec![SubCommandInfo { name: "subcmd", command: subcmd_info }],
        ..Default::default()
    }
}

#[cfg(test)]
#[test]
fn test_bash_generator() {
    let cmd = make_mock_command();
    let bash_out = crate::bash::Bash::generate("mycmd", &cmd);

    assert!(bash_out.contains("cmd=\"mycmd_subcmd\""));
    assert!(bash_out.contains("opts=\"--verbose -v\""));
    assert!(bash_out.contains("cmds=\"subcmd\""));
}

#[cfg(test)]
#[test]
fn test_zsh_generator() {
    let cmd = make_mock_command();
    let zsh_out = crate::zsh::Zsh::generate("mycmd", &cmd);

    assert!(zsh_out.contains("#compdef mycmd"));
    assert!(zsh_out.contains("'*::command:->subcmd'"));
    assert!(zsh_out.contains("(-v --verbose)")); // Simplified check
}

#[cfg(test)]
#[test]
fn test_fish_generator() {
    let cmd = make_mock_command();
    let fish_out = crate::fish::Fish::generate("mycmd", &cmd);

    assert!(fish_out.contains("complete -c mycmd -n 'not __fish_seen_subcommand_from subcmd' -f -l verbose -s v -d 'verbose output'"));
    assert!(fish_out.contains("complete -c mycmd -n 'not __fish_seen_subcommand_from subcmd' -f -a 'subcmd' -d 'a sub command'"));
}

#[cfg(test)]
#[test]
fn test_nushell_generator() {
    let cmd = make_mock_command();
    let nushell_out = crate::nushell::Nushell::generate("mycmd", &cmd);

    assert!(nushell_out.contains("export extern \"mycmd\" ["));
    assert!(nushell_out.contains("--verbose(-v) # verbose output"));
    assert!(nushell_out.contains("export extern \"mycmd subcmd\" ["));
}
