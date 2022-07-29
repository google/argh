// Copyright (c) 2022 Google LLC All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use crate::{EarlyExit, FromArgs, HelpInfo, HelpSubCommandInfo, HelpSubCommandsInfo};

/// Trait to expose the command line help information in JSON format.
///
pub trait Help: FromArgs {
    /// Help info extracted from the argh information provided by the
    /// FromArgs trait.
    const HELP_INFO: &'static HelpInfo;

    /// Returns a JSON encoded string of the usage information. This is intended to
    /// create a "machine readable" version of the help text to enable reference
    /// documentation generation.
    fn help_json_from_args(strs: &[&str]) -> Result<String, EarlyExit> {
        Ok(Self::HELP_INFO.help_json_from_args(strs))
    }
}

/// HelpSubCommands is used to store the Help information for
/// subcommands declared.
pub trait HelpSubCommands {
    /// Help info extracted from the argh information provided by the
    /// FromArgs trait.
    const HELP_INFO: &'static HelpSubCommandsInfo;
}

/// The help information for a single subcommand.
pub trait HelpSubCommand {
    /// Help info extracted from the argh information provided by the
    /// FromArgs trait.
    const HELP_INFO: &'static HelpSubCommandInfo;
}

impl<T: HelpSubCommand> HelpSubCommands for T {
    /// The HELPINFO is the collection of HelpSubCommand objects.
    const HELP_INFO: &'static HelpSubCommandsInfo =
        &HelpSubCommandsInfo { optional: false, commands: &[<T as HelpSubCommand>::HELP_INFO] };
}
