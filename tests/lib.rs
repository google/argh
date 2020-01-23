#![cfg(test)]
// Copyright (c) 2020 Google LLC All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.


use {argh::FromArgs, std::fmt::Debug};

#[test]
fn basic_example() {
    #[derive(FromArgs, PartialEq, Debug)]
    /// Reach new heights.
    struct GoUp {
        /// whether or not to jump
        #[argh(switch, short = 'j')]
        jump: bool,

        /// how high to go
        #[argh(option)]
        height: usize,

        /// an optional nickname for the pilot
        #[argh(option)]
        pilot_nickname: Option<String>,
    }

    let up = GoUp::from_args(&["cmdname"], &["--height", "5"]).expect("failed go_up");
    assert_eq!(up, GoUp { jump: false, height: 5, pilot_nickname: None });
}

#[test]
fn custom_from_str_example() {
    #[derive(FromArgs)]
    /// Goofy thing.
    struct FiveStruct {
        /// always five
        #[argh(option, from_str_fn(always_five))]
        five: usize,
    }

    fn always_five(_value: &str) -> Result<usize, String> {
        Ok(5)
    }

    let f = FiveStruct::from_args(&["cmdname"], &["--five", "woot"]).expect("failed to five");
    assert_eq!(f.five, 5);
}

#[test]
fn subcommand_example() {
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

    let one = TopLevel::from_args(&["cmdname"], &["one", "--x", "2"]).expect("sc 1");
    assert_eq!(one, TopLevel { nested: MySubCommandEnum::One(SubCommandOne { x: 2 }) },);

    let two = TopLevel::from_args(&["cmdname"], &["two", "--fooey"]).expect("sc 2");
    assert_eq!(two, TopLevel { nested: MySubCommandEnum::Two(SubCommandTwo { fooey: true }) },);
}

#[test]
fn multiline_doc_comment_description() {
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

    assert_help_string::<Cmd>(
        r###"Usage: test_arg_0 [--s]

Short description

Options:
  --s               a switch with a description that is spread across a number
                    of lines of comments.
  --help            display usage information
"###,
    );
}

#[test]
fn explicit_long_value_for_option() {
    #[derive(FromArgs, Debug)]
    /// Short description
    struct Cmd {
        #[argh(option, long = "foo")]
        /// bar bar
        x: u8,
    }

    let cmd = Cmd::from_args(&["cmdname"], &["--foo", "5"]).unwrap();
    assert_eq!(cmd.x, 5);
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
fn default_number() {
    #[derive(FromArgs)]
    /// Short description
    struct Cmd {
        #[argh(option, default = "5")]
        /// fooey
        x: u8,
    }

    let cmd = Cmd::from_args(&["cmdname"], &[]).unwrap();
    assert_eq!(cmd.x, 5);
}

#[test]
fn default_function() {
    const MSG: &str = "hey I just met you";
    fn call_me_maybe() -> String {
        MSG.to_string()
    }

    #[derive(FromArgs)]
    /// Short description
    struct Cmd {
        #[argh(option, default = "call_me_maybe()")]
        /// fooey
        msg: String,
    }

    let cmd = Cmd::from_args(&["cmdname"], &[]).unwrap();
    assert_eq!(cmd.msg, MSG);
}

#[test]
fn missing_option_value() {
    #[derive(FromArgs, Debug)]
    /// Short description
    struct Cmd {
        #[argh(option)]
        /// fooey
        msg: String,
    }

    let e = Cmd::from_args(&["cmdname"], &["--msg"])
        .expect_err("Parsing missing option value should fail");
    assert_eq!(e.output, "No value provided for option \'--msg\'.\n");
    assert!(e.status.is_err());
}

fn assert_help_string<T: FromArgs>(help_str: &str) {
    match T::from_args(&["test_arg_0"], &["--help"]) {
        Ok(_) => panic!("help was parsed as args"),
        Err(e) => {
            assert_eq!(help_str, e.output);
            e.status.expect("help returned an error");
        }
    }
}

fn assert_output<T: FromArgs + Debug + PartialEq>(args: &[&str], expected: T) {
    let t = T::from_args(&["cmd"], args).expect("failed to parse");
    assert_eq!(t, expected);
}

fn assert_error<T: FromArgs + Debug>(args: &[&str], err_msg: &str) {
    let e = T::from_args(&["cmd"], args).expect_err("unexpectedly succeeded parsing");
    assert_eq!(err_msg, e.output);
    e.status.expect_err("error had a positive status");
}

mod positional {
    use super::*;

    #[derive(FromArgs, Debug, PartialEq)]
    /// Woot
    struct LastRepeating {
        #[argh(positional)]
        /// fooey
        a: u32,
        #[argh(positional)]
        /// fooey
        b: Vec<String>,
    }

    #[test]
    fn repeating() {
        assert_output(&["5"], LastRepeating { a: 5, b: vec![] });
        assert_output(&["5", "foo"], LastRepeating { a: 5, b: vec!["foo".into()] });
        assert_output(
            &["5", "foo", "bar"],
            LastRepeating { a: 5, b: vec!["foo".into(), "bar".into()] },
        );
        assert_help_string::<LastRepeating>(
            r###"Usage: test_arg_0 <a> [<b...>]

Woot

Options:
  --help            display usage information
"###,
        );
    }

    #[derive(FromArgs, Debug, PartialEq)]
    /// Woot
    struct LastOptional {
        #[argh(positional)]
        /// fooey
        a: u32,
        #[argh(positional)]
        /// fooey
        b: Option<String>,
    }

    #[test]
    fn optional() {
        assert_output(&["5"], LastOptional { a: 5, b: None });
        assert_output(&["5", "6"], LastOptional { a: 5, b: Some("6".into()) });
        assert_error::<LastOptional>(&["5", "6", "7"], "Unrecognized argument: 7\n");
    }

    #[derive(FromArgs, Debug, PartialEq)]
    /// Woot
    struct LastDefaulted {
        #[argh(positional)]
        /// fooey
        a: u32,
        #[argh(positional, default = "5")]
        /// fooey
        b: u32,
    }

    #[test]
    fn defaulted() {
        assert_output(&["5"], LastDefaulted { a: 5, b: 5 });
        assert_output(&["5", "6"], LastDefaulted { a: 5, b: 6 });
        assert_error::<LastDefaulted>(&["5", "6", "7"], "Unrecognized argument: 7\n");
    }

    #[derive(FromArgs, Debug, PartialEq)]
    /// Woot
    struct LastRequired {
        #[argh(positional)]
        /// fooey
        a: u32,
        #[argh(positional)]
        /// fooey
        b: u32,
    }

    #[test]
    fn required() {
        assert_output(&["5", "6"], LastRequired { a: 5, b: 6 });
        assert_error::<LastRequired>(
            &[],
            r###"Required positional arguments not provided:
    a
    b
"###,
        );
        assert_error::<LastRequired>(
            &["5"],
            r###"Required positional arguments not provided:
    b
"###,
        );
    }

    #[derive(FromArgs, Debug, PartialEq)]
    /// Woot
    struct WithSubcommand {
        #[argh(positional)]
        /// fooey
        a: String,
        #[argh(subcommand)]
        /// fooey
        b: Subcommand,
        #[argh(positional)]
        /// fooey
        c: Vec<String>,
    }

    #[derive(FromArgs, Debug, PartialEq)]
    #[argh(subcommand, name = "a")]
    /// Subcommand of positional::WithSubcommand.
    struct Subcommand {
        #[argh(positional)]
        /// fooey
        a: String,
        #[argh(positional)]
        /// fooey
        b: Vec<String>,
    }

    #[test]
    fn mixed_with_subcommand() {
        assert_output(
            &["first", "a", "a"],
            WithSubcommand {
                a: "first".into(),
                b: Subcommand { a: "a".into(), b: vec![] },
                c: vec![],
            },
        );

        assert_error::<WithSubcommand>(
            &["a", "a", "a"],
            r###"Required positional arguments not provided:
    a
"###,
        );

        assert_output(
            &["1", "2", "3", "a", "b", "c"],
            WithSubcommand {
                a: "1".into(),
                b: Subcommand { a: "b".into(), b: vec!["c".into()] },
                c: vec!["2".into(), "3".into()],
            },
        );
    }
}

/// Tests derived from
/// https://fuchsia.dev/fuchsia-src/development/api/cli and
/// https://fuchsia.dev/fuchsia-src/development/api/cli_help
mod fuchsia_commandline_tools_rubric {
    use super::*;

    /// Tests for the three required command line argument types:
    /// - exact text
    /// - arguments
    /// - options (i.e. switches and keys)
    #[test]
    fn three_command_line_argument_types() {
        // TODO(cramertj) add support for exact text and positional arguments
    }

    /// A piece of exact text may be required or optional
    #[test]
    fn exact_text_required_and_optional() {
        // TODO(cramertj) add support for exact text
    }

    /// Arguments are like function parameters or slots for data.
    /// The order often matters.
    #[test]
    fn arguments_ordered() {
        // TODO(cramertj) add support for ordered positional arguments
    }

    /// If a single argument is repeated, order may not matter, e.g. `<files>...`
    #[test]
    fn arguments_unordered() {
        // TODO(cramertj) add support for repeated positional arguments
    }

    // Short argument names must use one dash and a single letter.
    // TODO(cramertj): this should be a compile-fail test

    // Short argument names are optional, but all choices are required to have a `--` option.
    // TODO(cramertj): this should be a compile-fail test

    // Numeric options, such as `-1` and `-2`, are not allowed.
    // TODO(cramertj): this should be a compile-fail test

    #[derive(FromArgs)]
    /// One switch.
    struct OneSwitch {
        #[argh(switch, short = 's')]
        /// just a switch
        switchy: bool,
    }

    /// The presence of a switch means the feature it represents is "on",
    /// while its absence means that it is "off".
    #[test]
    fn switch_on_when_present() {
        let on = OneSwitch::from_args(&["cmdname"], &["-s"]).expect("parsing on");
        assert!(on.switchy);

        let off = OneSwitch::from_args(&["cmdname"], &[]).expect("parsing off");
        assert!(!off.switchy);
    }

    #[derive(FromArgs, Debug)]
    /// Two Switches
    struct TwoSwitches {
        #[argh(switch, short = 'a')]
        /// a
        a: bool,
        #[argh(switch, short = 'b')]
        /// b
        b: bool,
    }

    /// Running switches together is not allowed
    #[test]
    fn switches_cannot_run_together() {
        TwoSwitches::from_args(&["cmdname"], &["-a", "-b"])
            .expect("parsing separate should succeed");
        TwoSwitches::from_args(&["cmdname"], &["-ab"]).expect_err("parsing together should fail");
    }

    #[derive(FromArgs, Debug)]
    /// One keyed option
    struct OneOption {
        #[argh(option)]
        /// some description
        foo: String,
    }

    /// Do not use an equals punctuation or similar to separate the key and value.
    #[test]
    fn keyed_no_equals() {
        OneOption::from_args(&["cmdname"], &["--foo", "bar"])
            .expect("Parsing option value as separate arg should succeed");

        let e = OneOption::from_args(&["cmdname"], &["--foo=bar"])
            .expect_err("Parsing option value using `=` should fail");
        assert_eq!(e.output, "Unrecognized argument: --foo=bar\n");
        assert!(e.status.is_err());
    }

    // Two dashes on their own indicates the end of options.
    // Subsequent values are given to the tool as-is.
    //
    // It's unclear exactly what "are given to the tool as-is" in means in this
    // context, so we provide a few options for handling `--`, with it being
    // an error by default.
    //
    // TODO(cramertj) implement some behavior for `--`

    /// Double-dash is treated as an error by default.
    #[test]
    fn double_dash_default_error() {}

    /// Double-dash can be ignored for later manual parsing.
    #[test]
    fn double_dash_ignore() {}

    /// Double-dash should be treated as the end of flags and optional arguments,
    /// and the remainder of the values should be treated purely as positional arguments,
    /// even when their syntax matches that of options. e.g. `foo -- -e` should be parsed
    /// as passing a single positional argument with the value `-e`.
    #[test]
    fn double_dash_positional() {}

    /// Double-dash can be parsed into an optional field using a provided
    /// `fn(&[&str]) -> Result<T, EarlyExit>`.
    #[test]
    fn double_dash_custom() {}

    /// Repeating switches may be used to apply more emphasis.
    /// A common example is increasing verbosity by passing more `-v` switches.
    #[test]
    fn switches_repeating() {
        #[derive(FromArgs, Debug)]
        /// A type for testing repeating `-v`
        struct CountVerbose {
            #[argh(switch, short = 'v')]
            /// increase the verbosity of the command.
            verbose: i128,
        }

        let cv = CountVerbose::from_args(&["cmdname"], &["-v", "-v", "-v"])
            .expect("Parsing verbose flags should succeed");
        assert_eq!(cv.verbose, 3);
    }

    // When a tool has many subcommands, it should also have a help subcommand
    // that displays help about the subcommands, e.g. `fx help build`.
    //
    // Elsewhere in the docs, it says the syntax `--help` is required, so we
    // interpret that to mean:
    //
    // - `help` should always be accepted as a "keyword" in place of the first
    //   positional argument for both the main command and subcommands.
    //
    // - If followed by the name of a subcommand it should forward to the
    //   `--help` of said subcommand, otherwise it will fall back to the
    //   help of the righmost command / subcommand.
    //
    // - `--help` will always consider itself the only meaningful argument to
    //   the rightmost command / subcommand, and any following arguments will
    //   be treated as an error.

    #[derive(FromArgs, Debug)]
    /// A type for testing `--help`/`help`
    struct HelpTopLevel {
        #[argh(subcommand)]
        sub: HelpFirstSub,
    }

    #[derive(FromArgs, Debug)]
    #[argh(subcommand, name = "first")]
    /// First subcommmand for testing `help`.
    struct HelpFirstSub {
        #[argh(subcommand)]
        sub: HelpSecondSub,
    }

    #[derive(FromArgs, Debug)]
    #[argh(subcommand, name = "second")]
    /// Second subcommand for testing `help`.
    struct HelpSecondSub {}

    fn expect_help(args: &[&str], expected_help_string: &str) {
        let e = HelpTopLevel::from_args(&["cmdname"], args).expect_err("should exit early");
        assert_eq!(expected_help_string, e.output);
        e.status.expect("help returned an error");
    }

    const MAIN_HELP_STRING: &str = r###"Usage: cmdname <command> [<args>]

A type for testing `--help`/`help`

Options:
  --help            display usage information

Commands:
  first             First subcommmand for testing `help`.
"###;

    const FIRST_HELP_STRING: &str = r###"Usage: cmdname first <command> [<args>]

First subcommmand for testing `help`.

Options:
  --help            display usage information

Commands:
  second            Second subcommand for testing `help`.
"###;

    const SECOND_HELP_STRING: &str = r###"Usage: cmdname first second

Second subcommand for testing `help`.

Options:
  --help            display usage information
"###;

    #[test]
    fn help_keyword_main() {
        expect_help(&["help"], MAIN_HELP_STRING)
    }

    #[test]
    fn help_keyword_with_following_subcommand() {
        expect_help(&["help", "first"], FIRST_HELP_STRING);
    }

    #[test]
    fn help_keyword_between_subcommands() {
        expect_help(&["first", "help", "second"], SECOND_HELP_STRING);
    }

    #[test]
    fn help_keyword_with_two_trailing_subcommands() {
        expect_help(&["help", "first", "second"], SECOND_HELP_STRING);
    }

    #[test]
    fn help_flag_main() {
        expect_help(&["--help"], MAIN_HELP_STRING);
    }

    #[test]
    fn help_flag_subcommand() {
        expect_help(&["first", "--help"], FIRST_HELP_STRING);
    }

    #[test]
    fn help_flag_trailing_arguments_are_an_error() {
        let e = OneOption::from_args(&["cmdname"], &["--help", "--foo", "bar"])
            .expect_err("should exit early");
        assert_eq!("Trailing arguments are not allowed after `help`.", e.output);
        e.status.expect_err("should be an error");
    }

    // Commandline tools are expected to support common switches:
    // --help
    // --quiet
    // --verbose
    // --version

    // help_is_supported (see above help_* tests)

    #[test]
    fn quiet_is_supported() {
        // TODO support quiet
    }

    #[test]
    fn verbose_is_supported() {
        // TODO support verbose
    }

    #[test]
    fn version_is_supported() {
        // TODO support version
    }

    #[test]
    fn quiet_is_not_supported_in_subcommands() {
        // TODO support quiet
    }

    #[test]
    fn verbose_is_not_supported_in_subcommands() {
        // TODO support verbose
    }

    #[test]
    fn version_is_not_supported_in_subcommands() {
        // TODO support version
    }

    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(
        description = "Destroy the contents of <file>.",
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

    #[test]
    fn example_parses_correctly() {
        let help_example = HelpExample::from_args(
            &["<<<arg0>>>"],
            &["-f", "--scribble", "fooey", "blow-up", "--safely"],
        )
        .unwrap();

        assert_eq!(
            help_example,
            HelpExample {
                force: true,
                scribble: "fooey".to_string(),
                really_really_really_long_name_for_pat: false,
                verbose: false,
                command: HelpExampleSubCommands::BlowUp(BlowUp { safely: true }),
            },
        );
    }

    #[test]
    fn example_errors_on_missing_required_option_and_missing_required_subcommand() {
        let exit = HelpExample::from_args(&["<<<arg0>>>"], &[]).unwrap_err();
        exit.status.unwrap_err();
        assert_eq!(
            exit.output,
            concat!(
                "Required options not provided:\n",
                "    --scribble\n",
                "One of the following subcommands must be present:\n",
                "    help\n",
                "    blow-up\n",
                "    grind\n",
            ),
        );
    }

    #[test]
    fn help_example() {
        assert_help_string::<HelpExample>(
            r###"Usage: test_arg_0 [-f] [--really-really-really-long-name-for-pat] -s <scribble> [-v] <command> [<args>]

Destroy the contents of <file>.

Options:
  -f, --force       force, ignore minor errors. This description is so long that
                    it wraps to the next line.
  --really-really-really-long-name-for-pat
                    documentation
  -s, --scribble    write <scribble> repeatedly
  -v, --verbose     say more. Defaults to $BLAST_VERBOSE.
  --help            display usage information

Commands:
  blow-up           explosively separate
  grind             make smaller by many small cuts

Examples:
  Scribble 'abc' and then run |grind|.
  $ test_arg_0 -s 'abc' grind old.txt taxes.cp

Notes:
  Use `test_arg_0 help <command>` for details on [<args>] for a subcommand.

Error codes:
  2 The blade is too dull.
  3 Out of fuel.
"###,
        );
    }
}
