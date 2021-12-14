#![cfg(test)]
// Copyright (c) 2020 Google LLC All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::path::PathBuf;
use {argh::FromArgs, std::fmt::Debug};

#[test]
fn help_json_test_subcommand() {
    #[derive(FromArgs, PartialEq, Debug)]
    /// Top-level command.
    struct TopLevel {
        #[argh(subcommand)]
        nested: MySubCommandEnum,
    }

    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(subcommand)]
    enum MySubCommandEnum {
        One(SubCommandOne),
        Two(SubCommandTwo),
    }

    #[derive(FromArgs, PartialEq, Debug)]
    /// First subcommand.
    #[argh(subcommand, name = "one")]
    struct SubCommandOne {
        #[argh(option)]
        /// how many x
        x: usize,
    }

    #[derive(FromArgs, PartialEq, Debug)]
    /// Second subcommand.
    #[argh(subcommand, name = "two")]
    struct SubCommandTwo {
        #[argh(switch)]
        /// whether to fooey
        fooey: bool,
    }

    assert_help_json_string::<TopLevel>(
        vec!["--help-json"],
        r###"{
"usage": "test_arg_0 <command> [<args>]",
"description": "Top-level command.",
"options": [{"short": "", "long": "--help", "description": "display usage information"},
{"short": "", "long": "--help-json", "description": "display usage information encoded in JSON"}],
"positional": [],
"examples": "",
"notes": "",
"error_codes": [],
"subcommands": [{"name": "one", "description": "First subcommand."},
{"name": "two", "description": "Second subcommand."}]
}
"###,
    );

    assert_help_json_string::<TopLevel>(
        vec!["one", "--help-json"],
        r###"{
"usage": "test_arg_0 one --x <x>",
"description": "First subcommand.",
"options": [{"short": "", "long": "--x", "description": "how many x"},
{"short": "", "long": "--help", "description": "display usage information"},
{"short": "", "long": "--help-json", "description": "display usage information encoded in JSON"}],
"positional": [],
"examples": "",
"notes": "",
"error_codes": [],
"subcommands": []
}
"###,
    );
}

#[test]
fn help_json_test_multiline_doc_comment() {
    #[derive(FromArgs)]
    /// Short description
    struct Cmd {
        #[argh(switch)]
        /// a switch with a description
        /// that is spread across
        /// a number of
        /// lines of comments.
        _s: bool,
    }
    assert_help_json_string::<Cmd>(
        vec!["--help-json"],
        r###"{
"usage": "test_arg_0 [--s]",
"description": "Short description",
"options": [{"short": "", "long": "--s", "description": "a switch with a description that is spread across a number of lines of comments."},
{"short": "", "long": "--help", "description": "display usage information"},
{"short": "", "long": "--help-json", "description": "display usage information encoded in JSON"}],
"positional": [],
"examples": "",
"notes": "",
"error_codes": [],
"subcommands": []
}
"###,
    );
}

#[test]
fn help_json_test_basic_args() {
    #[allow(dead_code)]
    #[derive(FromArgs)]
    /// Basic command args demonstrating multiple types and cardinality. "With quotes"
    struct Basic {
        /// should the power be on. "Quoted value" should work too.
        #[argh(switch)]
        power: bool,

        /// option that is required because of no default and not Option<>.
        #[argh(option, long = "required")]
        required_flag: String,

        /// optional speed if not specified it is None.
        #[argh(option, short = 's')]
        speed: Option<u8>,

        /// repeatable option.
        #[argh(option, arg_name = "url")]
        link: Vec<String>,
    }
    assert_help_json_string::<Basic>(
        vec!["--help-json"],
        r###"{
"usage": "test_arg_0 [--power] --required <required> [-s <speed>] [--link <url...>]",
"description": "Basic command args demonstrating multiple types and cardinality. \"With quotes\"",
"options": [{"short": "", "long": "--power", "description": "should the power be on. \"Quoted value\" should work too."},
{"short": "", "long": "--required", "description": "option that is required because of no default and not Option<>."},
{"short": "s", "long": "--speed", "description": "optional speed if not specified it is None."},
{"short": "", "long": "--link", "description": "repeatable option."},
{"short": "", "long": "--help", "description": "display usage information"},
{"short": "", "long": "--help-json", "description": "display usage information encoded in JSON"}],
"positional": [],
"examples": "",
"notes": "",
"error_codes": [],
"subcommands": []
}
"###,
    );
}

#[test]
fn help_json_test_positional_args() {
    #[allow(dead_code)]
    #[derive(FromArgs)]
    /// Command with positional args demonstrating. "With quotes"
    struct Positional {
        /// the "root" position.
        #[argh(positional, arg_name = "root")]
        root_value: String,

        /// trunk value
        #[argh(positional)]
        trunk: String,

        /// leaves. There can be many leaves.
        #[argh(positional)]
        leaves: Vec<String>,
    }
    assert_help_json_string::<Positional>(
        vec!["--help-json"],
        r###"{
"usage": "test_arg_0 <root> <trunk> [<leaves...>]",
"description": "Command with positional args demonstrating. \"With quotes\"",
"options": [{"short": "", "long": "--help", "description": "display usage information"},
{"short": "", "long": "--help-json", "description": "display usage information encoded in JSON"}],
"positional": [{"name": "root", "description": "the \"root\" position."},
{"name": "trunk", "description": "trunk value"},
{"name": "leaves", "description": "leaves. There can be many leaves."}],
"examples": "",
"notes": "",
"error_codes": [],
"subcommands": []
}
"###,
    );
}

#[test]
fn help_json_test_optional_positional_args() {
    #[allow(dead_code)]
    #[derive(FromArgs)]
    /// Command with positional args demonstrating last value is optional
    struct Positional {
        /// the "root" position.
        #[argh(positional, arg_name = "root")]
        root_value: String,

        /// trunk value
        #[argh(positional)]
        trunk: String,

        /// leaves. There can be many leaves.
        #[argh(positional)]
        leaves: Option<String>,
    }
    assert_help_json_string::<Positional>(
        vec!["--help-json"],
        r###"{
"usage": "test_arg_0 <root> <trunk> [<leaves>]",
"description": "Command with positional args demonstrating last value is optional",
"options": [{"short": "", "long": "--help", "description": "display usage information"},
{"short": "", "long": "--help-json", "description": "display usage information encoded in JSON"}],
"positional": [{"name": "root", "description": "the \"root\" position."},
{"name": "trunk", "description": "trunk value"},
{"name": "leaves", "description": "leaves. There can be many leaves."}],
"examples": "",
"notes": "",
"error_codes": [],
"subcommands": []
}
"###,
    );
}

#[test]
fn help_json_test_default_positional_args() {
    #[allow(dead_code)]
    #[derive(FromArgs)]
    /// Command with positional args demonstrating last value is defaulted.
    struct Positional {
        /// the "root" position.
        #[argh(positional, arg_name = "root")]
        root_value: String,

        /// trunk value
        #[argh(positional)]
        trunk: String,

        /// leaves. There can be many leaves.
        #[argh(positional, default = "String::from(\"hello\")")]
        leaves: String,
    }
    assert_help_json_string::<Positional>(
        vec!["--help-json"],
        r###"{
"usage": "test_arg_0 <root> <trunk> [<leaves>]",
"description": "Command with positional args demonstrating last value is defaulted.",
"options": [{"short": "", "long": "--help", "description": "display usage information"},
{"short": "", "long": "--help-json", "description": "display usage information encoded in JSON"}],
"positional": [{"name": "root", "description": "the \"root\" position."},
{"name": "trunk", "description": "trunk value"},
{"name": "leaves", "description": "leaves. There can be many leaves."}],
"examples": "",
"notes": "",
"error_codes": [],
"subcommands": []
}
"###,
    );
}

#[test]
fn help_json_test_notes_examples_errors() {
    #[allow(dead_code)]
    #[derive(FromArgs)]
    /// Command with Examples and usage Notes, including error codes.
    #[argh(
        note = r##"
    These usage notes appear for {command_name} and how to best use it.
    The formatting should be preserved.
    one
    two
    three then a blank
    
    and one last line with "quoted text"."##,
        example = r##"
    Use the command with 1 file:

    `{command_name} /path/to/file`

    Use it with a "wildcard":

    `{command_name} /path/to/*`

     a blank line
    
    and one last line with "quoted text"."##,
        error_code(0, "Success"),
        error_code(1, "General Error"),
        error_code(2, "Some error with \"quotes\"")
    )]
    struct NotesExamplesErrors {
        /// the "root" position.
        #[argh(positional, arg_name = "files")]
        fields: Vec<PathBuf>,
    }
    assert_help_json_string::<NotesExamplesErrors>(
        vec!["--help-json"],
        r###"{
"usage": "test_arg_0 [<files...>]",
"description": "Command with Examples and usage Notes, including error codes.",
"options": [{"short": "", "long": "--help", "description": "display usage information"},
{"short": "", "long": "--help-json", "description": "display usage information encoded in JSON"}],
"positional": [{"name": "files", "description": "the \"root\" position."}],
"examples": "\n    Use the command with 1 file:\n\n    `test_arg_0 /path/to/file`\n\n    Use it with a \"wildcard\":\n\n    `test_arg_0 /path/to/*`\n\n     a blank line\n    \n    and one last line with \"quoted text\".",
"notes": "\n    These usage notes appear for test_arg_0 and how to best use it.\n    The formatting should be preserved.\n    one\n    two\n    three then a blank\n    \n    and one last line with \"quoted text\".",
"error_codes": [{"name": "0", "description": "Success"},
{"name": "1", "description": "General Error"},
{"name": "2", "description": "Some error with \"quotes\""}],
"subcommands": []
}
"###,
    );
}

#[test]
fn help_json_test_subcommands() {
    #[allow(dead_code)]
    #[derive(FromArgs)]
    ///Top level command with "subcommands".
    struct TopLevel {
        /// show verbose output
        #[argh(switch)]
        verbose: bool,

        /// this doc comment does not appear anywhere.
        #[argh(subcommand)]
        cmd: SubcommandEnum,
    }

    #[derive(FromArgs)]
    #[argh(subcommand)]
    /// Doc comments for subcommand enums does not appear in the help text.
    enum SubcommandEnum {
        Command1(Command1Args),
        Command2(Command2Args),
        Command3(Command3Args),
    }

    /// Command1 args are used for Command1.
    #[allow(dead_code)]
    #[derive(FromArgs)]
    #[argh(subcommand, name = "one")]
    struct Command1Args {
        /// the "root" position.
        #[argh(positional, arg_name = "root")]
        root_value: String,

        /// trunk value
        #[argh(positional)]
        trunk: String,

        /// leaves. There can be many leaves.
        #[argh(positional, default = "String::from(\"hello\")")]
        leaves: String,
    }
    /// Command2 args are used for Command2.
    #[allow(dead_code)]
    #[derive(FromArgs)]
    #[argh(subcommand, name = "two")]
    struct Command2Args {
        /// should the power be on. "Quoted value" should work too.
        #[argh(switch)]
        power: bool,

        /// option that is required because of no default and not Option<>.
        #[argh(option, long = "required")]
        required_flag: String,

        /// optional speed if not specified it is None.
        #[argh(option, short = 's')]
        speed: Option<u8>,

        /// repeatable option.
        #[argh(option, arg_name = "url")]
        link: Vec<String>,
    }
    /// Command3 args are used for Command3 which has no options or arguments.
    #[derive(FromArgs)]
    #[argh(subcommand, name = "three")]
    struct Command3Args {}

    assert_help_json_string::<TopLevel>(
        vec!["--help-json"],
        r###"{
"usage": "test_arg_0 [--verbose] <command> [<args>]",
"description": "Top level command with \"subcommands\".",
"options": [{"short": "", "long": "--verbose", "description": "show verbose output"},
{"short": "", "long": "--help", "description": "display usage information"},
{"short": "", "long": "--help-json", "description": "display usage information encoded in JSON"}],
"positional": [],
"examples": "",
"notes": "",
"error_codes": [],
"subcommands": [{"name": "one", "description": "Command1 args are used for Command1."},
{"name": "two", "description": "Command2 args are used for Command2."},
{"name": "three", "description": "Command3 args are used for Command3 which has no options or arguments."}]
}
"###,
    );

    assert_help_json_string::<TopLevel>(
        vec!["one", "--help-json"],
        r###"{
"usage": "test_arg_0 one <root> <trunk> [<leaves>]",
"description": "Command1 args are used for Command1.",
"options": [{"short": "", "long": "--help", "description": "display usage information"},
{"short": "", "long": "--help-json", "description": "display usage information encoded in JSON"}],
"positional": [{"name": "root", "description": "the \"root\" position."},
{"name": "trunk", "description": "trunk value"},
{"name": "leaves", "description": "leaves. There can be many leaves."}],
"examples": "",
"notes": "",
"error_codes": [],
"subcommands": []
}
"###,
    );

    assert_help_json_string::<TopLevel>(
        vec!["two", "--help-json"],
        r###"{
"usage": "test_arg_0 two [--power] --required <required> [-s <speed>] [--link <url...>]",
"description": "Command2 args are used for Command2.",
"options": [{"short": "", "long": "--power", "description": "should the power be on. \"Quoted value\" should work too."},
{"short": "", "long": "--required", "description": "option that is required because of no default and not Option<>."},
{"short": "s", "long": "--speed", "description": "optional speed if not specified it is None."},
{"short": "", "long": "--link", "description": "repeatable option."},
{"short": "", "long": "--help", "description": "display usage information"},
{"short": "", "long": "--help-json", "description": "display usage information encoded in JSON"}],
"positional": [],
"examples": "",
"notes": "",
"error_codes": [],
"subcommands": []
}
"###,
    );

    assert_help_json_string::<TopLevel>(
        vec!["three", "--help-json"],
        r###"{
"usage": "test_arg_0 three",
"description": "Command3 args are used for Command3 which has no options or arguments.",
"options": [{"short": "", "long": "--help", "description": "display usage information"},
{"short": "", "long": "--help-json", "description": "display usage information encoded in JSON"}],
"positional": [],
"examples": "",
"notes": "",
"error_codes": [],
"subcommands": []
}
"###,
    );
}

#[test]
fn help_json_test_subcommand_notes_examples() {
    #[allow(dead_code)]
    #[derive(FromArgs)]
    ///Top level command with "subcommands".
    #[argh(
        note = "Top level note",
        example = "Top level example",
        error_code(0, "Top level success")
    )]
    struct TopLevel {
        /// show verbose output
        #[argh(switch)]
        verbose: bool,

        /// this doc comment does not appear anywhere.
        #[argh(subcommand)]
        cmd: SubcommandEnum,
    }

    #[derive(FromArgs)]
    #[argh(subcommand)]
    /// Doc comments for subcommand enums does not appear in the help text.
    enum SubcommandEnum {
        Command1(Command1Args),
    }

    /// Command1 args are used for subcommand one.
    #[allow(dead_code)]
    #[derive(FromArgs)]
    #[argh(
        subcommand,
        name = "one",
        note = "{command_name} is used as a subcommand of \"Top level\"",
        example = "\"Typical\" usage is `{command_name}`.",
        error_code(0, "one level success")
    )]
    struct Command1Args {
        /// the "root" position.
        #[argh(positional, arg_name = "root")]
        root_value: String,

        /// trunk value
        #[argh(positional)]
        trunk: String,

        /// leaves. There can be many leaves.
        #[argh(positional, default = "String::from(\"hello\")")]
        leaves: String,
    }

    assert_help_json_string::<TopLevel>(
        vec!["--help-json"],
        r###"{
"usage": "test_arg_0 [--verbose] <command> [<args>]",
"description": "Top level command with \"subcommands\".",
"options": [{"short": "", "long": "--verbose", "description": "show verbose output"},
{"short": "", "long": "--help", "description": "display usage information"},
{"short": "", "long": "--help-json", "description": "display usage information encoded in JSON"}],
"positional": [],
"examples": "Top level example",
"notes": "Top level note",
"error_codes": [{"name": "0", "description": "Top level success"}],
"subcommands": [{"name": "one", "description": "Command1 args are used for subcommand one."}]
}
"###,
    );

    assert_help_json_string::<TopLevel>(
        vec!["one", "--help-json"],
        r###"{
"usage": "test_arg_0 one <root> <trunk> [<leaves>]",
"description": "Command1 args are used for subcommand one.",
"options": [{"short": "", "long": "--help", "description": "display usage information"},
{"short": "", "long": "--help-json", "description": "display usage information encoded in JSON"}],
"positional": [{"name": "root", "description": "the \"root\" position."},
{"name": "trunk", "description": "trunk value"},
{"name": "leaves", "description": "leaves. There can be many leaves."}],
"examples": "\"Typical\" usage is `test_arg_0 one`.",
"notes": "test_arg_0 one is used as a subcommand of \"Top level\"",
"error_codes": [{"name": "0", "description": "one level success"}],
"subcommands": []
}
"###,
    );
}

/// Test that descriptions can start with an initialism despite
/// usually being required to start with a lowercase letter.
#[derive(FromArgs)]
#[allow(unused)]
struct DescriptionStartsWithInitialism {
    /// URL fooey
    #[argh(option)]
    x: u8,
}

#[test]
fn help_json_test_example() {
    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(
        description = "Destroy the contents of <file> with a specific \"method of destruction\".",
        example = "Scribble 'abc' and then run |grind|.\n$ {command_name} -s 'abc' grind old.txt taxes.cp",
        note = "Use `{command_name} help <command>` for details on [<args>] for a subcommand.",
        error_code(2, "The blade is too dull."),
        error_code(3, "Out of fuel.")
    )]
    struct HelpExample {
        /// force, ignore minor errors. This description is so long that it wraps to the next line.
        #[argh(switch, short = 'f')]
        force: bool,

        /// documentation
        #[argh(switch)]
        really_really_really_long_name_for_pat: bool,

        /// write <scribble> repeatedly
        #[argh(option, short = 's')]
        scribble: String,

        /// say more. Defaults to $BLAST_VERBOSE.
        #[argh(switch, short = 'v')]
        verbose: bool,

        #[argh(subcommand)]
        command: HelpExampleSubCommands,
    }

    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(subcommand)]
    enum HelpExampleSubCommands {
        BlowUp(BlowUp),
        Grind(GrindCommand),
    }

    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(subcommand, name = "blow-up")]
    /// explosively separate
    struct BlowUp {
        /// blow up bombs safely
        #[argh(switch)]
        safely: bool,
    }

    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(subcommand, name = "grind", description = "make smaller by many small cuts")]
    struct GrindCommand {
        /// wear a visor while grinding
        #[argh(switch)]
        safely: bool,
    }

    assert_help_json_string::<HelpExample>(
        vec!["--help-json"],
        r###"{
"usage": "test_arg_0 [-f] [--really-really-really-long-name-for-pat] -s <scribble> [-v] <command> [<args>]",
"description": "Destroy the contents of <file> with a specific \"method of destruction\".",
"options": [{"short": "f", "long": "--force", "description": "force, ignore minor errors. This description is so long that it wraps to the next line."},
{"short": "", "long": "--really-really-really-long-name-for-pat", "description": "documentation"},
{"short": "s", "long": "--scribble", "description": "write <scribble> repeatedly"},
{"short": "v", "long": "--verbose", "description": "say more. Defaults to $BLAST_VERBOSE."},
{"short": "", "long": "--help", "description": "display usage information"},
{"short": "", "long": "--help-json", "description": "display usage information encoded in JSON"}],
"positional": [],
"examples": "Scribble 'abc' and then run |grind|.\n$ test_arg_0 -s 'abc' grind old.txt taxes.cp",
"notes": "Use `test_arg_0 help <command>` for details on [<args>] for a subcommand.",
"error_codes": [{"name": "2", "description": "The blade is too dull."},
{"name": "3", "description": "Out of fuel."}],
"subcommands": [{"name": "blow-up", "description": "explosively separate"},
{"name": "grind", "description": "make smaller by many small cuts"}]
}
"###,
    );
}

fn assert_help_json_string<T: FromArgs>(args: Vec<&str>, help_str: &str) {
    match T::from_args(&["test_arg_0"], &args) {
        Ok(_) => panic!("help-json was parsed as args"),
        Err(e) => {
            assert_eq!(help_str, e.output);
            e.status.expect("help-json returned an error");
        }
    }
}
