#![cfg(test)]
// Copyright (c) 2020 Google LLC All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::path::PathBuf;

use {argh:: {FromArgs, Help}, std::fmt::Debug};

#[test]
fn help_json_test_subcommand() {
    #[derive(FromArgs, Help, PartialEq, Debug)]
    /// Top-level command.
    struct TopLevel {
        #[argh(subcommand)]
        nested: MySubCommandEnum,
    }

    #[derive(FromArgs, Help, PartialEq, Debug)]
    #[argh(subcommand)]
    enum MySubCommandEnum {
        One(SubCommandOne),
        Two(SubCommandTwo),
    }

    #[derive(FromArgs, Help, PartialEq, Debug)]
    /// First subcommand.
    #[argh(subcommand, name = "one")]
    struct SubCommandOne {
        #[argh(option)]
        /// how many x
        x: usize,
    }

    #[derive(FromArgs, Help, PartialEq, Debug)]
    /// Second subcommand.
    #[argh(subcommand, name = "two")]
    struct SubCommandTwo {
        #[argh(switch)]
        /// whether to fooey
        fooey: bool,
    }

    assert_help_json_string::<TopLevel>(
        vec![],
        r###"{
"name": "test_arg_0",
"usage": "test_arg_0 <command> [<args>]",
"description": "Top-level command.",
"flags": [{"short": "", "long": "--help", "description": "display usage information", "arg_name": "", "optionality": "optional"}],
"positional": [],
"examples": "",
"notes": "",
"error_codes": [],
"subcommands": [{
"name": "one",
"usage": "test_arg_0 one --x <x>",
"description": "First subcommand.",
"flags": [{"short": "", "long": "--x", "description": "how many x", "arg_name": "x", "optionality": "required"},
{"short": "", "long": "--help", "description": "display usage information", "arg_name": "", "optionality": "optional"}],
"positional": [],
"examples": "",
"notes": "",
"error_codes": [],
"subcommands": []
}
,
{
"name": "two",
"usage": "test_arg_0 two [--fooey]",
"description": "Second subcommand.",
"flags": [{"short": "", "long": "--fooey", "description": "whether to fooey", "arg_name": "", "optionality": "optional"},
{"short": "", "long": "--help", "description": "display usage information", "arg_name": "", "optionality": "optional"}],
"positional": [],
"examples": "",
"notes": "",
"error_codes": [],
"subcommands": []
}
]
}
"###,
    );

    assert_help_json_string::<SubCommandOne>(
        vec!["one"],
        r###"{
"name": "one",
"usage": "test_arg_0 one --x <x>",
"description": "First subcommand.",
"flags": [{"short": "", "long": "--x", "description": "how many x", "arg_name": "x", "optionality": "required"},
{"short": "", "long": "--help", "description": "display usage information", "arg_name": "", "optionality": "optional"}],
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
    #[derive(FromArgs, Help)]
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
        vec![],
        r###"{
"name": "test_arg_0",
"usage": "test_arg_0 [--s]",
"description": "Short description",
"flags": [{"short": "", "long": "--s", "description": "a switch with a description that is spread across a number of lines of comments.", "arg_name": "", "optionality": "optional"},
{"short": "", "long": "--help", "description": "display usage information", "arg_name": "", "optionality": "optional"}],
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
    #[derive(FromArgs, Help)]
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
        vec![],
        r###"{
"name": "test_arg_0",
"usage": "test_arg_0 [--power] --required <required> [-s <speed>] [--link <url...>]",
"description": "Basic command args demonstrating multiple types and cardinality. \"With quotes\"",
"flags": [{"short": "", "long": "--power", "description": "should the power be on. \"Quoted value\" should work too.", "arg_name": "", "optionality": "optional"},
{"short": "", "long": "--required", "description": "option that is required because of no default and not Option<>.", "arg_name": "required", "optionality": "required"},
{"short": "s", "long": "--speed", "description": "optional speed if not specified it is None.", "arg_name": "speed", "optionality": "optional"},
{"short": "", "long": "--link", "description": "repeatable option.", "arg_name": "url", "optionality": "repeating"},
{"short": "", "long": "--help", "description": "display usage information", "arg_name": "", "optionality": "optional"}],
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
    #[derive(FromArgs, Help)]
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
        vec![],
        r###"{
"name": "test_arg_0",
"usage": "test_arg_0 <root> <trunk> [<leaves...>]",
"description": "Command with positional args demonstrating. \"With quotes\"",
"flags": [{"short": "", "long": "--help", "description": "display usage information", "arg_name": "", "optionality": "optional"}],
"positional": [{"name": "root", "description": "the \"root\" position.", "optionality": "required"},
{"name": "trunk", "description": "trunk value", "optionality": "required"},
{"name": "leaves", "description": "leaves. There can be many leaves.", "optionality": "repeating"}],
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
    #[derive(FromArgs, Help)]
    /// Command with positional args demonstrating last value is optional
    struct Positional {
        /// the "root" position.
        #[argh(positional, arg_name = "root")]
        root_value: String,

        /// trunk value
        #[argh(positional)]
        trunk: String,

        /// leaves. There can be an optional leaves.
        #[argh(positional)]
        leaves: Option<String>,
    }
    assert_help_json_string::<Positional>(
        vec![],
        r###"{
"name": "test_arg_0",
"usage": "test_arg_0 <root> <trunk> [<leaves>]",
"description": "Command with positional args demonstrating last value is optional",
"flags": [{"short": "", "long": "--help", "description": "display usage information", "arg_name": "", "optionality": "optional"}],
"positional": [{"name": "root", "description": "the \"root\" position.", "optionality": "required"},
{"name": "trunk", "description": "trunk value", "optionality": "required"},
{"name": "leaves", "description": "leaves. There can be an optional leaves.", "optionality": "optional"}],
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
    #[derive(FromArgs, Help)]
    /// Command with positional args demonstrating last value is defaulted.
    struct Positional {
        /// the "root" position.
        #[argh(positional, arg_name = "root")]
        root_value: String,

        /// trunk value
        #[argh(positional)]
        trunk: String,

        /// leaves. There can be one leaf, defaults to hello.
        #[argh(positional, default = "String::from(\"hello\")")]
        leaves: String,
    }
    assert_help_json_string::<Positional>(
        vec![],
        r###"{
"name": "test_arg_0",
"usage": "test_arg_0 <root> <trunk> [<leaves>]",
"description": "Command with positional args demonstrating last value is defaulted.",
"flags": [{"short": "", "long": "--help", "description": "display usage information", "arg_name": "", "optionality": "optional"}],
"positional": [{"name": "root", "description": "the \"root\" position.", "optionality": "required"},
{"name": "trunk", "description": "trunk value", "optionality": "required"},
{"name": "leaves", "description": "leaves. There can be one leaf, defaults to hello.", "optionality": "optional"}],
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
    #[derive(FromArgs, Help)]
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
        vec![],
        r###"{
"name": "test_arg_0",
"usage": "test_arg_0 [<files...>]",
"description": "Command with Examples and usage Notes, including error codes.",
"flags": [{"short": "", "long": "--help", "description": "display usage information", "arg_name": "", "optionality": "optional"}],
"positional": [{"name": "files", "description": "the \"root\" position.", "optionality": "repeating"}],
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
    #[derive(FromArgs, Help)]
    ///Top level command with "subcommands".
    struct TopLevel {
        /// show verbose output
        #[argh(switch)]
        verbose: bool,

        /// this doc comment does not appear anywhere.
        #[argh(subcommand)]
        cmd: SubcommandEnum,
    }

    #[derive(FromArgs, Help)]
    #[argh(subcommand)]
    /// Doc comments for subcommand enums does not appear in the help text.
    enum SubcommandEnum {
        Command1(Command1Args),
        Command2(Command2Args),
        Command3(Command3Args),
    }

    /// Command1 args are used for Command1.
    #[allow(dead_code)]
    #[derive(FromArgs, Help)]
    #[argh(subcommand, name = "one")]
    struct Command1Args {
        /// the "root" position.
        #[argh(positional, arg_name = "root")]
        root_value: String,

        /// trunk value
        #[argh(positional)]
        trunk: String,

        /// leaves. There can be zero leaves, defaults to hello.
        #[argh(positional, default = "String::from(\"hello\")")]
        leaves: String,
    }
    /// Command2 args are used for Command2.
    #[allow(dead_code)]
    #[derive(FromArgs, Help)]
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
    #[derive(FromArgs, Help)]
    #[argh(subcommand, name = "three")]
    struct Command3Args {}

    assert_help_json_string::<TopLevel>(
        vec![],
        r###"{
"name": "test_arg_0",
"usage": "test_arg_0 [--verbose] <command> [<args>]",
"description": "Top level command with \"subcommands\".",
"flags": [{"short": "", "long": "--verbose", "description": "show verbose output", "arg_name": "", "optionality": "optional"},
{"short": "", "long": "--help", "description": "display usage information", "arg_name": "", "optionality": "optional"}],
"positional": [],
"examples": "",
"notes": "",
"error_codes": [],
"subcommands": [{
"name": "one",
"usage": "test_arg_0 one <root> <trunk> [<leaves>]",
"description": "Command1 args are used for Command1.",
"flags": [{"short": "", "long": "--help", "description": "display usage information", "arg_name": "", "optionality": "optional"}],
"positional": [{"name": "root", "description": "the \"root\" position.", "optionality": "required"},
{"name": "trunk", "description": "trunk value", "optionality": "required"},
{"name": "leaves", "description": "leaves. There can be zero leaves, defaults to hello.", "optionality": "optional"}],
"examples": "",
"notes": "",
"error_codes": [],
"subcommands": []
}
,
{
"name": "two",
"usage": "test_arg_0 two [--power] --required <required> [-s <speed>] [--link <url...>]",
"description": "Command2 args are used for Command2.",
"flags": [{"short": "", "long": "--power", "description": "should the power be on. \"Quoted value\" should work too.", "arg_name": "", "optionality": "optional"},
{"short": "", "long": "--required", "description": "option that is required because of no default and not Option<>.", "arg_name": "required", "optionality": "required"},
{"short": "s", "long": "--speed", "description": "optional speed if not specified it is None.", "arg_name": "speed", "optionality": "optional"},
{"short": "", "long": "--link", "description": "repeatable option.", "arg_name": "url", "optionality": "repeating"},
{"short": "", "long": "--help", "description": "display usage information", "arg_name": "", "optionality": "optional"}],
"positional": [],
"examples": "",
"notes": "",
"error_codes": [],
"subcommands": []
}
,
{
"name": "three",
"usage": "test_arg_0 three",
"description": "Command3 args are used for Command3 which has no options or arguments.",
"flags": [{"short": "", "long": "--help", "description": "display usage information", "arg_name": "", "optionality": "optional"}],
"positional": [],
"examples": "",
"notes": "",
"error_codes": [],
"subcommands": []
}
]
}
"###,
    );
}

#[test]
fn help_json_test_subcommand_notes_examples() {
    #[allow(dead_code)]
    #[derive(FromArgs, Help)]
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

    #[derive(FromArgs, Help)]
    #[argh(subcommand)]
    /// Doc comments for subcommand enums does not appear in the help text.
    enum SubcommandEnum {
        Command1(Command1Args),
    }

    /// Command1 args are used for subcommand one.
    #[allow(dead_code)]
    #[derive(FromArgs, Help)]
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
        vec![],
        r###"{
"name": "test_arg_0",
"usage": "test_arg_0 [--verbose] <command> [<args>]",
"description": "Top level command with \"subcommands\".",
"flags": [{"short": "", "long": "--verbose", "description": "show verbose output", "arg_name": "", "optionality": "optional"},
{"short": "", "long": "--help", "description": "display usage information", "arg_name": "", "optionality": "optional"}],
"positional": [],
"examples": "Top level example",
"notes": "Top level note",
"error_codes": [{"name": "0", "description": "Top level success"}],
"subcommands": [{
"name": "one",
"usage": "test_arg_0 one <root> <trunk> [<leaves>]",
"description": "Command1 args are used for subcommand one.",
"flags": [{"short": "", "long": "--help", "description": "display usage information", "arg_name": "", "optionality": "optional"}],
"positional": [{"name": "root", "description": "the \"root\" position.", "optionality": "required"},
{"name": "trunk", "description": "trunk value", "optionality": "required"},
{"name": "leaves", "description": "leaves. There can be many leaves.", "optionality": "optional"}],
"examples": "\"Typical\" usage is `test_arg_0 one`.",
"notes": "test_arg_0 one is used as a subcommand of \"Top level\"",
"error_codes": [{"name": "0", "description": "one level success"}],
"subcommands": []
}
]
}
"###,
    );

    assert_help_json_string::<Command1Args>(
        vec!["one"],
        r###"{
"name": "one",
"usage": "test_arg_0 one <root> <trunk> [<leaves>]",
"description": "Command1 args are used for subcommand one.",
"flags": [{"short": "", "long": "--help", "description": "display usage information", "arg_name": "", "optionality": "optional"}],
"positional": [{"name": "root", "description": "the \"root\" position.", "optionality": "required"},
{"name": "trunk", "description": "trunk value", "optionality": "required"},
{"name": "leaves", "description": "leaves. There can be many leaves.", "optionality": "optional"}],
"examples": "\"Typical\" usage is `test_arg_0 one`.",
"notes": "test_arg_0 one is used as a subcommand of \"Top level\"",
"error_codes": [{"name": "0", "description": "one level success"}],
"subcommands": []
}
"###,
    );
}

#[test]
fn help_json_test_example() {
    #[derive(FromArgs, Help, PartialEq, Debug)]
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

    #[derive(FromArgs, Help, PartialEq, Debug)]
    #[argh(subcommand)]
    enum HelpExampleSubCommands {
        BlowUp(BlowUp),
        Grind(GrindCommand),
    }

    #[derive(FromArgs,Help, PartialEq, Debug)]
    #[argh(subcommand, name = "blow-up")]
    /// explosively separate
    struct BlowUp {
        /// blow up bombs safely
        #[argh(switch)]
        safely: bool,
    }

    #[derive(FromArgs, Help, PartialEq, Debug)]
    #[argh(subcommand, name = "grind", description = "make smaller by many small cuts")]
    struct GrindCommand {
        /// wear a visor while grinding
        #[argh(switch)]
        safely: bool,
    }

    assert_help_json_string::<HelpExample>(
        vec![],
        r###"{
"name": "test_arg_0",
"usage": "test_arg_0 [-f] [--really-really-really-long-name-for-pat] -s <scribble> [-v] <command> [<args>]",
"description": "Destroy the contents of <file> with a specific \"method of destruction\".",
"flags": [{"short": "f", "long": "--force", "description": "force, ignore minor errors. This description is so long that it wraps to the next line.", "arg_name": "", "optionality": "optional"},
{"short": "", "long": "--really-really-really-long-name-for-pat", "description": "documentation", "arg_name": "", "optionality": "optional"},
{"short": "s", "long": "--scribble", "description": "write <scribble> repeatedly", "arg_name": "scribble", "optionality": "required"},
{"short": "v", "long": "--verbose", "description": "say more. Defaults to $BLAST_VERBOSE.", "arg_name": "", "optionality": "optional"},
{"short": "", "long": "--help", "description": "display usage information", "arg_name": "", "optionality": "optional"}],
"positional": [],
"examples": "Scribble 'abc' and then run |grind|.\n$ test_arg_0 -s 'abc' grind old.txt taxes.cp",
"notes": "Use `test_arg_0 help <command>` for details on [<args>] for a subcommand.",
"error_codes": [{"name": "2", "description": "The blade is too dull."},
{"name": "3", "description": "Out of fuel."}],
"subcommands": [{
"name": "blow-up",
"usage": "test_arg_0 blow-up [--safely]",
"description": "explosively separate",
"flags": [{"short": "", "long": "--safely", "description": "blow up bombs safely", "arg_name": "", "optionality": "optional"},
{"short": "", "long": "--help", "description": "display usage information", "arg_name": "", "optionality": "optional"}],
"positional": [],
"examples": "",
"notes": "",
"error_codes": [],
"subcommands": []
}
,
{
"name": "grind",
"usage": "test_arg_0 grind [--safely]",
"description": "make smaller by many small cuts",
"flags": [{"short": "", "long": "--safely", "description": "wear a visor while grinding", "arg_name": "", "optionality": "optional"},
{"short": "", "long": "--help", "description": "display usage information", "arg_name": "", "optionality": "optional"}],
"positional": [],
"examples": "",
"notes": "",
"error_codes": [],
"subcommands": []
}
]
}
"###,
    );
}

/*
{
\"name\": \"test_arg_0\",
\"usage\": \"test_arg_0 [-f] [--really-really-really-long-name-for-pat] -s <scribble> [-v] <command> [<args>]\",
\"description\": \"Destroy the contents of <file> with a specific \\\"method of destruction\\\".\",
\"flags\": [{\"short\": \"f\", \"long\": \"--force\", \"description\": \"force, ignore minor errors. This description is so long that it wraps to the next line.\", \"arg_name\": \"\", \"optionality\": \"optional\"},
{\"short\": \"\", \"long\": \"--really-really-really-long-name-for-pat\", \"description\": \"documentation\", \"arg_name\": \"\", \"optionality\": \"optional\"},
{\"short\": \"s\", \"long\": \"--scribble\", \"description\": \"write <scribble> repeatedly\", \"arg_name\": \"scribble\", \"optionality\": \"required\"},
{\"short\": \"v\", \"long\": \"--verbose\", \"description\": \"say more. Defaults to $BLAST_VERBOSE.\", \"arg_name\": \"\", \"optionality\": \"optional\"},
{\"short\": \"\", \"long\": \"--help\", \"description\": \"display usage information\", \"arg_name\": \"\", \"optionality\": \"optional\"}]
,\n\"positional\": [],
\"examples\": \"Scribble 'abc' and then run |grind|.\\n$ test_arg_0 -s 'abc' grind old.txt taxes.cp\",\n\"notes\": \"Use `test_arg_0 help <command>` for details on [<args>] for a subcommand.\",
\"error_codes\": [{\"name\": \"2\", \"description\": \"The blade is too dull.\", \"optionality\": \"\"},\n{\"name\": \"3\", \"description\": \"Out of fuel.\", \"optionality\": \"\"}],
\"subcommands\": [{\n\"name\": \"blow-up\",\n\"usage\": \"test_arg_0 blow-up [--safely]\",
\"description\": \"explosively separate\",
\"flags\": [{\"short\": \"\", \"long\": \"--safely\", \"description\": \"blow up bombs safely\", \"arg_name\": \"\", \"optionality\": \"optional\"},
{\"short\": \"\", \"long\": \"--help\", \"description\": \"display usage information\", \"arg_name\": \"\", \"optionality\": \"optional\"}],
\"positional\": [],
\"examples\": \"\",
\"notes\": \"\",
\"error_codes\": [],\n\"subcommands\": []\n}\n,\n{
    \"name\": \"grind\",\n\"usage\": \"test_arg_0 grind [--safely]\",\n\"description\": \"make smaller by many small cuts\",
    \"flags\": [{\"short\": \"\", \"long\": \"--safely\", \"description\": \"wear a visor while grinding\", \"arg_name\": \"\", \"optionality\": \"optional\"},\n{\"short\": \"\", \"long\": \"--help\", \"description\": \"display usage information\", \"arg_name\": \"\", \"optionality\": \"optional\"}],
\"positional\": [],\n\"examples\": \"\",\n\"notes\": \"\",\n\"error_codes\": [],\n\"subcommands\": []\n}\n]\n}

*/

fn assert_help_json_string<T: Help>(args: Vec<&str>, help_str: &str) {
    let mut command_args = vec!["test_arg_0"];
    command_args.extend_from_slice(&args);
    let actual_value = T::help_json_from_args(&command_args)
        .expect("unexpected error getting help_json_from_args");
    assert_eq!(help_str, actual_value)
}