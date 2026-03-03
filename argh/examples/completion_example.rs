// Copyright (c) 2026 Google LLC All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use argh::{ArgsInfo, FromArgs};
use argh_complete::Generator;

#[derive(FromArgs, ArgsInfo)]
/// An example command showing off autocompletion generation.
struct MyCmd {
    #[argh(subcommand)]
    cmd: Subcommands,
}

#[derive(FromArgs, ArgsInfo)]
#[argh(subcommand)]
enum Subcommands {
    Completion(CompletionCmd),
    DoThings(DoThingsCmd),
}

#[derive(FromArgs, ArgsInfo)]
/// Generate shell completions.
#[argh(subcommand, name = "completion")]
struct CompletionCmd {
    /// the shell to generate for (bash, zsh, fish)
    #[argh(positional)]
    shell: String,
}

#[derive(FromArgs, ArgsInfo)]
/// Do some things.
#[argh(subcommand, name = "do-things")]
struct DoThingsCmd {
    /// how many things to do
    #[argh(option, short = 'n', default = "5")]
    count: usize,

    /// do it quickly
    #[argh(switch, short = 'q')]
    quick: bool,
}

fn main() {
    let args: MyCmd = argh::from_env();

    match args.cmd {
        Subcommands::Completion(cmd) => {
            let cmd_info = MyCmd::get_args_info();
            let mut command_name = String::new();
            if let Some(arg0) = std::env::args().next() {
                command_name = std::path::Path::new(&arg0)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
            }
            if command_name.is_empty() {
                command_name = cmd_info.name.to_string();
            }
            match cmd.shell.as_str() {
                "bash" => {
                    println!("{}", argh_complete::bash::Bash::generate(&command_name, &cmd_info))
                }
                "zsh" => {
                    println!("{}", argh_complete::zsh::Zsh::generate(&command_name, &cmd_info))
                }
                "fish" => {
                    println!("{}", argh_complete::fish::Fish::generate(&command_name, &cmd_info))
                }
                _ => eprintln!("Unsupported shell: {}", cmd.shell),
            }
        }
        Subcommands::DoThings(cmd) => {
            println!("Doing {} things (quick: {})", cmd.count, cmd.quick);
        }
    }
}
