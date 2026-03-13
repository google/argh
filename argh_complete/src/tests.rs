// Copyright (c) 2026 Google LLC All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use crate::Generator;
use argh_shared::{CommandInfoWithArgs, FlagInfo, FlagInfoKind, Optionality, SubCommandInfo};

fn make_mock_command() -> CommandInfoWithArgs<'static> {
    let subcmd_test_list =
        CommandInfoWithArgs { name: "list", description: "a list command", ..Default::default() };
    let subcmd_test_run =
        CommandInfoWithArgs { name: "run", description: "a run command", ..Default::default() };
    let subcmd_test = CommandInfoWithArgs {
        name: "test",
        description: "a test command inside subcmd",
        commands: vec![
            SubCommandInfo { name: "list", command: subcmd_test_list },
            SubCommandInfo { name: "run", command: subcmd_test_run },
        ],
        ..Default::default()
    };
    let subcmd_help =
        CommandInfoWithArgs { name: "help", description: "a help command", ..Default::default() };

    let subcmd_info = CommandInfoWithArgs {
        name: "subcmd",
        description: "a sub command",
        commands: vec![
            SubCommandInfo { name: "help", command: subcmd_help },
            SubCommandInfo { name: "test", command: subcmd_test },
        ],
        ..Default::default()
    };

    let test_run_info =
        CommandInfoWithArgs { name: "run", description: "a run command", ..Default::default() };
    let test_build_info =
        CommandInfoWithArgs { name: "build", description: "a build command", ..Default::default() };
    let subcmd_2info = CommandInfoWithArgs {
        name: "test",
        description: "a test command",
        commands: vec![
            SubCommandInfo { name: "run", command: test_run_info },
            SubCommandInfo { name: "build", command: test_build_info },
        ],
        ..Default::default()
    };

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
        commands: vec![
            SubCommandInfo { name: "subcmd", command: subcmd_info },
            SubCommandInfo { name: "test", command: subcmd_2info },
        ],
        ..Default::default()
    }
}

#[test]
fn test_bash_generator() {
    let cmd = make_mock_command();
    let bash_out = crate::bash::Bash::generate("mycmd", &cmd);

    assert!(bash_out.contains("cmd=\"mycmd_subcmd\""));
    assert!(bash_out.contains("opts=\"--verbose -v\""));
    assert!(bash_out.contains("cmds=\"help test\""), "bash should have updated cmds for subcmd");
}

#[test]
fn test_zsh_generator() {
    let cmd = make_mock_command();
    let zsh_out = crate::zsh::Zsh::generate("mycmd", &cmd);

    assert!(zsh_out.contains("#compdef mycmd"));
    assert!(zsh_out.contains("'*::command:->subcmd'"));
    assert!(zsh_out.contains("(-v --verbose)")); // Simplified check
}

#[test]
fn test_fish_generator() {
    let cmd = make_mock_command();
    let fish_out = crate::fish::Fish::generate("mycmd", &cmd);

    assert!(fish_out.contains("function __fish_mycmd_using_command"));
    assert!(fish_out.contains(
        "complete -c mycmd -n '__fish_mycmd_using_command' -f -l verbose -s v -d 'verbose output'"
    ));
    assert!(fish_out.contains(
        "complete -c mycmd -n '__fish_mycmd_using_command' -f -a 'subcmd' -d 'a sub command'"
    ));
}

#[test]
fn test_nushell_generator() {
    let cmd = make_mock_command();
    let nushell_out = crate::nushell::Nushell::generate("mycmd", &cmd);

    assert!(nushell_out.contains("export extern \"mycmd\" ["));
    assert!(nushell_out.contains("--verbose(-v) # verbose output"));
    assert!(nushell_out.contains("export extern \"mycmd subcmd\" ["));
}
