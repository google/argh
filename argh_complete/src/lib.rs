// Copyright (c) 2026 Google LLC All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//! Autocompletion generators for `argh`-based CLIs.

pub mod bash;
pub mod fish;
pub mod nushell;
pub mod zsh;

use argh_shared::CommandInfoWithArgs;

/// A trait for generating shell completions.
pub trait Generator {
    /// Generates the completion script for the given command name and structure.
    fn generate(cmd_name: &str, cmd: &CommandInfoWithArgs<'_>) -> String;
}

#[cfg(test)]
mod tests;
