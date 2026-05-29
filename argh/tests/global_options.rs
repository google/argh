// Copyright (c) 2026 Google LLC All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Test that #[argh(global)] allows parent command options after subcommands

use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(global)]
/// Top-level command with global option placement
struct TopCommand {
    #[argh(switch, short = 'v')]
    /// verbose mode
    verbose: bool,

    #[argh(option, short = 'c')]
    /// config file
    config: Option<String>,

    #[argh(subcommand)]
    sub: SubCmd,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum SubCmd {
    Run(RunCmd),
    Build(BuildCmd),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "run")]
/// Run the program
struct RunCmd {
    #[argh(option, short = 'n')]
    /// number of iterations
    iterations: usize,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "build")]
/// Build the program
struct BuildCmd {
    #[argh(switch, short = 'r')]
    /// release mode
    release: bool,
}

fn parse_from(args: &[&str]) -> Result<TopCommand, argh::EarlyExit> {
    TopCommand::from_args(&["test"], args)
}

#[test]
fn global_option_after_subcommand() {
    let result = parse_from(&["run", "-n", "5", "--verbose"]).unwrap();
    assert!(result.verbose);
    assert_eq!(result.sub, SubCmd::Run(RunCmd { iterations: 5 }));
}

#[test]
fn global_option_before_subcommand() {
    let result = parse_from(&["--verbose", "run", "-n", "5"]).unwrap();
    assert!(result.verbose);
    assert_eq!(result.sub, SubCmd::Run(RunCmd { iterations: 5 }));
}

#[test]
fn global_multiple_options_after() {
    let result = parse_from(&["run", "-n", "5", "--verbose", "-c", "config.toml"]).unwrap();
    assert!(result.verbose);
    assert_eq!(result.config, Some("config.toml".to_string()));
    assert_eq!(result.sub, SubCmd::Run(RunCmd { iterations: 5 }));
}

#[test]
fn global_mixed_order() {
    let result = parse_from(&["--verbose", "run", "-n", "5", "-c", "config.toml"]).unwrap();
    assert!(result.verbose);
    assert_eq!(result.config, Some("config.toml".to_string()));
    assert_eq!(result.sub, SubCmd::Run(RunCmd { iterations: 5 }));
}

#[test]
fn global_short_option_after() {
    let result = parse_from(&["run", "-n", "5", "-v"]).unwrap();
    assert!(result.verbose);
    assert_eq!(result.sub, SubCmd::Run(RunCmd { iterations: 5 }));
}

#[test]
fn global_with_build_subcommand() {
    let result = parse_from(&["build", "--release", "--verbose"]).unwrap();
    assert!(result.verbose);
    assert_eq!(result.sub, SubCmd::Build(BuildCmd { release: true }));
}

// Test that non-global commands still reject options after subcommands
#[derive(FromArgs, PartialEq, Debug)]
/// Non-global command
struct NonGlobalTop {
    #[argh(switch)]
    /// verbose mode
    verbose: bool,

    #[argh(subcommand)]
    sub: NonGlobalSub,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum NonGlobalSub {
    Run(NonGlobalRun),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "run")]
/// Run command
struct NonGlobalRun {
    #[argh(option)]
    /// iterations
    iterations: usize,
}

#[test]
fn non_global_rejects_option_after_subcommand() {
    let result = NonGlobalTop::from_args(&["test"], &["run", "--iterations", "5", "--verbose"]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.output.contains("Unrecognized"));
}

#[test]
fn non_global_accepts_option_before_subcommand() {
    let result =
        NonGlobalTop::from_args(&["test"], &["--verbose", "run", "--iterations", "5"]).unwrap();
    assert_eq!(result.verbose, true);
}
