// Copyright (c) 2020 Google LLC All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//! Shared functionality between argh_derive and the argh runtime.
//!
//! This library is intended only for internal use by these two crates.

/// Information about a particular command used for output.
pub struct CommandInfo<'a> {
    /// The name of the command.
    pub name: &'a str,
    /// A short description of the command's functionality.
    pub description: &'a str,
}

pub const INDENT: &str = "  ";
const DESCRIPTION_INDENT: usize = 20;
const WRAP_WIDTH: usize = 80;

/// Write command names and descriptions to an output string.
pub fn write_description(out: &mut String, cmd: &CommandInfo<'_>) {
    let mut current_line = INDENT.to_string();
    current_line.push_str(cmd.name);

    if cmd.description.is_empty() {
        new_line(&mut current_line, out);
        return;
    }

    if !indent_description(&mut current_line) {
        // Start the description on a new line if the flag names already
        // add up to more than DESCRIPTION_INDENT.
        new_line(&mut current_line, out);
    }

    let mut words = cmd.description.split(' ').peekable();
    while let Some(first_word) = words.next() {
        indent_description(&mut current_line);
        current_line.push_str(first_word);

        'inner: while let Some(&word) = words.peek() {
            if (char_len(&current_line) + char_len(word) + 1) > WRAP_WIDTH {
                new_line(&mut current_line, out);
                break 'inner;
            } else {
                // advance the iterator
                let _ = words.next();
                current_line.push(' ');
                current_line.push_str(word);
            }
        }
    }
    new_line(&mut current_line, out);
}

// Indent the current line in to DESCRIPTION_INDENT chars.
// Returns a boolean indicating whether or not spacing was added.
fn indent_description(line: &mut String) -> bool {
    let cur_len = char_len(line);
    if cur_len < DESCRIPTION_INDENT {
        let num_spaces = DESCRIPTION_INDENT - cur_len;
        line.extend(std::iter::repeat(' ').take(num_spaces));
        true
    } else {
        false
    }
}

fn char_len(s: &str) -> usize {
    s.chars().count()
}

// Append a newline and the current line to the output,
// clearing the current line.
fn new_line(current_line: &mut String, out: &mut String) {
    out.push('\n');
    out.push_str(current_line);
    current_line.truncate(0);
}
