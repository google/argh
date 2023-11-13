// Copyright (c) 2020 Google LLC All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Deny a bunch of uncommon clippy lints to make sure the generated code won't trigger a warning.
#![deny(
    clippy::indexing_slicing,
    clippy::panic_in_result_fn,
    clippy::str_to_string,
    clippy::unreachable,
    clippy::unwrap_in_result
)]

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
fn generic_example() {
    use std::fmt::Display;
    use std::str::FromStr;

    #[derive(FromArgs, PartialEq, Debug)]
    /// Reach new heights.
    struct GoUp<S: FromStr>
    where
        <S as FromStr>::Err: Display,
    {
        /// whether or not to jump
        #[argh(switch, short = 'j')]
        jump: bool,

        /// how high to go
        #[argh(option)]
        height: usize,

        /// an optional nickname for the pilot
        #[argh(option)]
        pilot_nickname: Option<S>,
    }

    let up = GoUp::<String>::from_args(&["cmdname"], &["--height", "5"]).expect("failed go_up");
    assert_eq!(up, GoUp::<String> { jump: false, height: 5, pilot_nickname: None });
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
fn help_trigger_example() {
    /// Height options
    #[derive(FromArgs)]
    #[argh(help_triggers("-h", "--help", "help"))]
    struct Height {
        /// how high to go
        #[argh(option)]
        _height: usize,
    }

    assert_help_string::<Height>(
        r#"Usage: test_arg_0 --height <height>

Height options

Options:
  --height          how high to go
  -h, --help, help  display usage information
"#,
    );
}

#[test]
fn nested_from_str_example() {
    #[derive(FromArgs)]
    /// Goofy thing.
    struct FiveStruct {
        /// always five
        #[argh(option, from_str_fn(nested::always_five))]
        five: usize,
    }

    pub mod nested {
        pub fn always_five(_value: &str) -> Result<usize, String> {
            Ok(5)
        }
    }

    let f = FiveStruct::from_args(&["cmdname"], &["--five", "woot"]).expect("failed to five");
    assert_eq!(f.five, 5);
}

#[test]
fn method_from_str_example() {
    #[derive(FromArgs)]
    /// Goofy thing.
    struct FiveStruct {
        /// always five
        #[argh(option, from_str_fn(AlwaysFive::<usize>::always_five))]
        five: usize,
    }

    struct AlwaysFive<T>(T);

    impl AlwaysFive<usize> {
        fn always_five(_value: &str) -> Result<usize, String> {
            Ok(5)
        }
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
fn dynamic_subcommand_example() {
    #[derive(PartialEq, Debug)]
    struct DynamicSubCommandImpl {
        got: String,
    }

    impl argh::DynamicSubCommand for DynamicSubCommandImpl {
        fn commands() -> &'static [&'static argh::CommandInfo] {
            &[
                &argh::CommandInfo { name: "three", description: "Third command" },
                &argh::CommandInfo { name: "four", description: "Fourth command" },
                &argh::CommandInfo { name: "five", description: "Fifth command" },
            ]
        }

        fn try_redact_arg_values(
            _command_name: &[&str],
            _args: &[&str],
        ) -> Option<Result<Vec<String>, argh::EarlyExit>> {
            Some(Err(argh::EarlyExit::from("Test should not redact".to_owned())))
        }

        fn try_from_args(
            command_name: &[&str],
            args: &[&str],
        ) -> Option<Result<DynamicSubCommandImpl, argh::EarlyExit>> {
            let command_name = match command_name.last() {
                Some(x) => *x,
                None => return Some(Err(argh::EarlyExit::from("No command".to_owned()))),
            };
            let description = Self::commands().iter().find(|x| x.name == command_name)?.description;
            if args.len() > 1 {
                Some(Err(argh::EarlyExit::from("Too many arguments".to_owned())))
            } else if let Some(arg) = args.first() {
                Some(Ok(DynamicSubCommandImpl { got: format!("{} got {:?}", description, arg) }))
            } else {
                Some(Err(argh::EarlyExit::from("Not enough arguments".to_owned())))
            }
        }
    }

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
        #[argh(dynamic)]
        ThreeFourFive(DynamicSubCommandImpl),
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

    let three = TopLevel::from_args(&["cmdname"], &["three", "beans"]).expect("sc 3");
    assert_eq!(
        three,
        TopLevel {
            nested: MySubCommandEnum::ThreeFourFive(DynamicSubCommandImpl {
                got: "Third command got \"beans\"".to_owned()
            })
        },
    );

    let four = TopLevel::from_args(&["cmdname"], &["four", "boulders"]).expect("sc 4");
    assert_eq!(
        four,
        TopLevel {
            nested: MySubCommandEnum::ThreeFourFive(DynamicSubCommandImpl {
                got: "Fourth command got \"boulders\"".to_owned()
            })
        },
    );

    let five = TopLevel::from_args(&["cmdname"], &["five", "gold rings"]).expect("sc 5");
    assert_eq!(
        five,
        TopLevel {
            nested: MySubCommandEnum::ThreeFourFive(DynamicSubCommandImpl {
                got: "Fifth command got \"gold rings\"".to_owned()
            })
        },
    );
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
  --help, help      display usage information
"###,
    );
}

#[test]
fn escaped_doc_comment_description() {
    #[derive(FromArgs)]
    /// A \description\:
    /// \!\"\#\$\%\&\'\(\)\*\+\,\-\.\/\:\;\<\=\>\?\@\[\\\]\^\_\`\{\|\}\~\
    struct Cmd {
        #[argh(switch)]
        /// a \description\:
        /// \!\"\#\$\%\&\'\(\)\*\+\,\-\.\/\:\;\<\=\>\?\@\[\\\]\^\_\`\{\|\}\~\
        _s: bool,
    }

    assert_help_string::<Cmd>(
        r###"Usage: test_arg_0 [--s]

A \description: !"#$%&'()*+,-./:;<=>?@[\]^_`{|}~\

Options:
  --s               a \description: !"#$%&'()*+,-./:;<=>?@[\]^_`{|}~\
  --help, help      display usage information
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
        MSG.to_owned()
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
        _msg: String,
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

mod options {
    use super::*;

    #[derive(argh::FromArgs, Debug, PartialEq)]
    /// Woot
    struct Parsed {
        #[argh(option, short = 'n')]
        /// fooey
        n: usize,
    }

    #[test]
    fn parsed() {
        assert_output(&["-n", "5"], Parsed { n: 5 });
        assert_error::<Parsed>(
            &["-n", "x"],
            r###"Error parsing option '-n' with value 'x': invalid digit found in string
"###,
        );
    }

    #[derive(argh::FromArgs, Debug, PartialEq)]
    /// Woot
    struct Repeating {
        #[argh(option, short = 'n')]
        /// fooey
        n: Vec<String>,
    }

    #[test]
    fn repeating() {
        assert_help_string::<Repeating>(
            r###"Usage: test_arg_0 [-n <n...>]

Woot

Options:
  -n, --n           fooey
  --help, help      display usage information
"###,
        );
    }

    #[derive(argh::FromArgs, Debug, PartialEq)]
    /// Woot
    struct WithArgName {
        #[argh(option, arg_name = "name")]
        /// fooey
        option_name: Option<String>,
    }

    #[test]
    fn with_arg_name() {
        assert_help_string::<WithArgName>(
            r###"Usage: test_arg_0 [--option-name <name>]

Woot

Options:
  --option-name     fooey
  --help, help      display usage information
"###,
        );
    }
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

Positional Arguments:
  a                 fooey
  b                 fooey

Options:
  --help, help      display usage information
"###,
        );
    }

    #[derive(FromArgs, Debug, PartialEq)]
    /// Woot
    struct LastRepeatingGreedy {
        #[argh(positional)]
        /// fooey
        a: u32,
        #[argh(switch)]
        /// woo
        b: bool,
        #[argh(option)]
        /// stuff
        c: Option<String>,
        #[argh(positional, greedy)]
        /// fooey
        d: Vec<String>,
    }

    #[test]
    fn positional_greedy() {
        assert_output(&["5"], LastRepeatingGreedy { a: 5, b: false, c: None, d: vec![] });
        assert_output(
            &["5", "foo"],
            LastRepeatingGreedy { a: 5, b: false, c: None, d: vec!["foo".into()] },
        );
        assert_output(
            &["5", "foo", "bar"],
            LastRepeatingGreedy { a: 5, b: false, c: None, d: vec!["foo".into(), "bar".into()] },
        );
        assert_output(
            &["5", "--b", "foo", "bar"],
            LastRepeatingGreedy { a: 5, b: true, c: None, d: vec!["foo".into(), "bar".into()] },
        );
        assert_output(
            &["5", "foo", "bar", "--b"],
            LastRepeatingGreedy {
                a: 5,
                b: false,
                c: None,
                d: vec!["foo".into(), "bar".into(), "--b".into()],
            },
        );
        assert_output(
            &["5", "--c", "hi", "foo", "bar"],
            LastRepeatingGreedy {
                a: 5,
                b: false,
                c: Some("hi".into()),
                d: vec!["foo".into(), "bar".into()],
            },
        );
        assert_output(
            &["5", "foo", "bar", "--c", "hi"],
            LastRepeatingGreedy {
                a: 5,
                b: false,
                c: None,
                d: vec!["foo".into(), "bar".into(), "--c".into(), "hi".into()],
            },
        );
        assert_output(
            &["5", "foo", "bar", "--", "hi"],
            LastRepeatingGreedy {
                a: 5,
                b: false,
                c: None,
                d: vec!["foo".into(), "bar".into(), "--".into(), "hi".into()],
            },
        );
        assert_help_string::<LastRepeatingGreedy>(
            r###"Usage: test_arg_0 <a> [--b] [--c <c>] [d...]

Woot

Positional Arguments:
  a                 fooey

Options:
  --b               woo
  --c               stuff
  --help, help      display usage information
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

    #[derive(argh::FromArgs, Debug, PartialEq)]
    /// Woot
    struct Parsed {
        #[argh(positional)]
        /// fooey
        n: usize,
    }

    #[test]
    fn parsed() {
        assert_output(&["5"], Parsed { n: 5 });
        assert_error::<Parsed>(
            &["x"],
            r###"Error parsing positional argument 'n' with value 'x': invalid digit found in string
"###,
        );
    }

    #[derive(FromArgs, Debug, PartialEq)]
    /// Woot
    struct WithOption {
        #[argh(positional)]
        /// fooey
        a: String,
        #[argh(option)]
        /// fooey
        b: String,
    }

    #[test]
    fn mixed_with_option() {
        assert_output(&["first", "--b", "foo"], WithOption { a: "first".into(), b: "foo".into() });

        assert_error::<WithOption>(
            &[],
            r###"Required positional arguments not provided:
    a
Required options not provided:
    --b
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

    #[derive(FromArgs, Debug, PartialEq)]
    /// Woot
    struct Underscores {
        #[argh(positional)]
        /// fooey
        a_: String,
    }

    #[test]
    fn positional_name_with_underscores() {
        assert_output(&["first"], Underscores { a_: "first".into() });

        assert_error::<Underscores>(
            &[],
            r###"Required positional arguments not provided:
    a
"###,
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
        _a: bool,
        #[argh(switch, short = 'b')]
        /// b
        _b: bool,
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
        _foo: String,
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
    fn double_dash_positional() {
        #[derive(FromArgs, Debug, PartialEq)]
        /// Positional arguments list
        struct StringList {
            #[argh(positional)]
            /// a list of strings
            strs: Vec<String>,

            #[argh(switch)]
            /// some flag
            flag: bool,
        }

        assert_output(
            &["--", "a", "-b", "--flag"],
            StringList { strs: vec!["a".into(), "-b".into(), "--flag".into()], flag: false },
        );
        assert_output(
            &["--flag", "--", "-a", "b"],
            StringList { strs: vec!["-a".into(), "b".into()], flag: true },
        );
        assert_output(&["--", "--help"], StringList { strs: vec!["--help".into()], flag: false });
        assert_output(
            &["--", "-a", "--help"],
            StringList { strs: vec!["-a".into(), "--help".into()], flag: false },
        );
    }

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
        _sub: HelpFirstSub,
    }

    #[derive(FromArgs, Debug)]
    #[argh(subcommand, name = "first")]
    /// First subcommmand for testing `help`.
    struct HelpFirstSub {
        #[argh(subcommand)]
        _sub: HelpSecondSub,
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
  --help, help      display usage information

Commands:
  first             First subcommmand for testing `help`.
"###;

    const FIRST_HELP_STRING: &str = r###"Usage: cmdname first <command> [<args>]

First subcommmand for testing `help`.

Options:
  --help, help      display usage information

Commands:
  second            Second subcommand for testing `help`.
"###;

    const SECOND_HELP_STRING: &str = r###"Usage: cmdname first second

Second subcommand for testing `help`.

Options:
  --help, help      display usage information
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
        #[argh(dynamic)]
        Plugin(HelpExamplePlugin),
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

    #[derive(PartialEq, Debug)]
    struct HelpExamplePlugin {
        got: String,
    }

    impl argh::DynamicSubCommand for HelpExamplePlugin {
        fn commands() -> &'static [&'static argh::CommandInfo] {
            &[&argh::CommandInfo { name: "plugin", description: "Example dynamic command" }]
        }

        fn try_redact_arg_values(
            _command_name: &[&str],
            _args: &[&str],
        ) -> Option<Result<Vec<String>, argh::EarlyExit>> {
            Some(Err(argh::EarlyExit::from("Test should not redact".to_owned())))
        }

        fn try_from_args(
            command_name: &[&str],
            args: &[&str],
        ) -> Option<Result<HelpExamplePlugin, argh::EarlyExit>> {
            if command_name.last() != Some(&"plugin") {
                None
            } else if args.len() > 1 {
                Some(Err(argh::EarlyExit::from("Too many arguments".to_owned())))
            } else if let Some(arg) = args.first() {
                Some(Ok(HelpExamplePlugin { got: format!("plugin got {:?}", arg) }))
            } else {
                Some(Ok(HelpExamplePlugin { got: "plugin got no argument".to_owned() }))
            }
        }
    }

    #[test]
    fn example_parses_correctly() {
        let help_example = HelpExample::from_args(
            &["program-name"],
            &["-f", "--scribble", "fooey", "blow-up", "--safely"],
        )
        .unwrap();

        assert_eq!(
            help_example,
            HelpExample {
                force: true,
                scribble: "fooey".to_owned(),
                really_really_really_long_name_for_pat: false,
                verbose: false,
                command: HelpExampleSubCommands::BlowUp(BlowUp { safely: true }),
            },
        );
    }

    #[test]
    fn example_errors_on_missing_required_option_and_missing_required_subcommand() {
        let exit = HelpExample::from_args(&["program-name"], &[]).unwrap_err();
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
                "    plugin\n",
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
  --help, help      display usage information

Commands:
  blow-up           explosively separate
  grind             make smaller by many small cuts
  plugin            Example dynamic command

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

    #[allow(dead_code)]
    #[derive(argh::FromArgs)]
    /// Destroy the contents of <file>.
    struct WithArgName {
        #[argh(positional, arg_name = "name")]
        username: String,
    }

    #[test]
    fn with_arg_name() {
        assert_help_string::<WithArgName>(
            r###"Usage: test_arg_0 <name>

Destroy the contents of <file>.

Positional Arguments:
  name

Options:
  --help, help      display usage information
"###,
        );
    }

    #[test]
    fn hidden_help_attribute() {
        #[derive(FromArgs)]
        /// Short description
        struct Cmd {
            /// this one should be hidden
            #[argh(positional, hidden_help)]
            _one: String,
            #[argh(positional)]
            /// this one is real
            _two: String,
            /// this one should be hidden
            #[argh(option, hidden_help)]
            _three: String,
        }

        assert_help_string::<Cmd>(
            r###"Usage: test_arg_0 <two>

Short description

Positional Arguments:
  two               this one is real

Options:
  --help, help      display usage information
"###,
        );
    }
}

#[test]
fn redact_arg_values_no_args() {
    #[derive(FromArgs, Debug)]
    /// Short description
    struct Cmd {
        #[argh(option)]
        /// a msg param
        _msg: Option<String>,
    }

    let actual = Cmd::redact_arg_values(&["program-name"], &[]).unwrap();
    assert_eq!(actual, &["program-name"]);
}

#[test]
fn redact_arg_values_optional_arg() {
    #[derive(FromArgs, Debug)]
    /// Short description
    struct Cmd {
        #[argh(option)]
        /// a msg param
        _msg: Option<String>,
    }

    let actual = Cmd::redact_arg_values(&["program-name"], &["--msg", "hello"]).unwrap();
    assert_eq!(actual, &["program-name", "--msg"]);
}

#[test]
fn redact_arg_values_optional_arg_short() {
    #[derive(FromArgs, Debug)]
    /// Short description
    struct Cmd {
        #[argh(option, short = 'm')]
        /// a msg param
        _msg: Option<String>,
    }

    let actual = Cmd::redact_arg_values(&["program-name"], &["-m", "hello"]).unwrap();
    assert_eq!(actual, &["program-name", "-m"]);
}

#[test]
fn redact_arg_values_optional_arg_long() {
    #[derive(FromArgs, Debug)]
    /// Short description
    struct Cmd {
        #[argh(option, long = "my-msg")]
        /// a msg param
        _msg: Option<String>,
    }

    let actual = Cmd::redact_arg_values(&["program-name"], &["--my-msg", "hello"]).unwrap();
    assert_eq!(actual, &["program-name", "--my-msg"]);
}

#[test]
fn redact_arg_values_two_option_args() {
    #[derive(FromArgs, Debug)]
    /// Short description
    struct Cmd {
        #[argh(option)]
        /// a msg param
        _msg: String,

        #[argh(option)]
        /// a delivery param
        _delivery: String,
    }

    let actual =
        Cmd::redact_arg_values(&["program-name"], &["--msg", "hello", "--delivery", "next day"])
            .unwrap();
    assert_eq!(actual, &["program-name", "--msg", "--delivery"]);
}

#[test]
fn redact_arg_values_option_one_optional_args() {
    #[derive(FromArgs, Debug)]
    /// Short description
    struct Cmd {
        #[argh(option)]
        /// a msg param
        _msg: String,

        #[argh(option)]
        /// a delivery param
        _delivery: Option<String>,
    }

    let actual =
        Cmd::redact_arg_values(&["program-name"], &["--msg", "hello", "--delivery", "next day"])
            .unwrap();
    assert_eq!(actual, &["program-name", "--msg", "--delivery"]);

    let actual = Cmd::redact_arg_values(&["program-name"], &["--msg", "hello"]).unwrap();
    assert_eq!(actual, &["program-name", "--msg"]);
}

#[test]
fn redact_arg_values_option_repeating() {
    #[derive(FromArgs, Debug)]
    /// Short description
    struct Cmd {
        #[argh(option)]
        /// fooey
        _msg: Vec<String>,
    }

    let actual = Cmd::redact_arg_values(&["program-name"], &[]).unwrap();
    assert_eq!(actual, &["program-name"]);

    let actual =
        Cmd::redact_arg_values(&["program-name"], &["--msg", "abc", "--msg", "xyz"]).unwrap();
    assert_eq!(actual, &["program-name", "--msg", "--msg"]);
}

#[test]
fn redact_arg_values_switch() {
    #[derive(FromArgs, Debug)]
    /// Short description
    struct Cmd {
        #[argh(switch, short = 'f')]
        /// speed of cmd
        _faster: bool,
    }

    let actual = Cmd::redact_arg_values(&["program-name"], &["--faster"]).unwrap();
    assert_eq!(actual, &["program-name", "--faster"]);

    let actual = Cmd::redact_arg_values(&["program-name"], &["-f"]).unwrap();
    assert_eq!(actual, &["program-name", "-f"]);
}

#[test]
fn redact_arg_values_positional() {
    #[derive(FromArgs, Debug)]
    /// Short description
    struct Cmd {
        #[allow(unused)]
        #[argh(positional)]
        /// speed of cmd
        speed: u8,
    }

    let actual = Cmd::redact_arg_values(&["program-name"], &["5"]).unwrap();
    assert_eq!(actual, &["program-name", "speed"]);
}

#[test]
fn redact_arg_values_positional_arg_name() {
    #[derive(FromArgs, Debug)]
    /// Short description
    struct Cmd {
        #[argh(positional, arg_name = "speed")]
        /// speed of cmd
        _speed: u8,
    }

    let actual = Cmd::redact_arg_values(&["program-name"], &["5"]).unwrap();
    assert_eq!(actual, &["program-name", "speed"]);
}

#[test]
fn redact_arg_values_positional_repeating() {
    #[derive(FromArgs, Debug)]
    /// Short description
    struct Cmd {
        #[argh(positional, arg_name = "speed")]
        /// speed of cmd
        _speed: Vec<u8>,
    }

    let actual = Cmd::redact_arg_values(&["program-name"], &["5", "6"]).unwrap();
    assert_eq!(actual, &["program-name", "speed", "speed"]);
}

#[test]
fn redact_arg_values_positional_err() {
    #[derive(FromArgs, Debug)]
    /// Short description
    struct Cmd {
        #[argh(positional, arg_name = "speed")]
        /// speed of cmd
        _speed: u8,
    }

    let actual = Cmd::redact_arg_values(&["program-name"], &[]).unwrap_err();
    assert_eq!(
        actual,
        argh::EarlyExit {
            output: "Required positional arguments not provided:\n    speed\n".into(),
            status: Err(()),
        }
    );
}

#[test]
fn redact_arg_values_two_positional() {
    #[derive(FromArgs, Debug)]
    /// Short description
    struct Cmd {
        #[argh(positional, arg_name = "speed")]
        /// speed of cmd
        _speed: u8,

        #[argh(positional, arg_name = "direction")]
        /// direction
        _direction: String,
    }

    let actual = Cmd::redact_arg_values(&["program-name"], &["5", "north"]).unwrap();
    assert_eq!(actual, &["program-name", "speed", "direction"]);
}

#[test]
fn redact_arg_values_positional_option() {
    #[derive(FromArgs, Debug)]
    /// Short description
    struct Cmd {
        #[argh(positional, arg_name = "speed")]
        /// speed of cmd
        _speed: u8,

        #[argh(option)]
        /// direction
        _direction: String,
    }

    let actual = Cmd::redact_arg_values(&["program-name"], &["5", "--direction", "north"]).unwrap();
    assert_eq!(actual, &["program-name", "speed", "--direction"]);
}

#[test]
fn redact_arg_values_positional_optional_option() {
    #[derive(FromArgs, Debug)]
    /// Short description
    struct Cmd {
        #[argh(positional, arg_name = "speed")]
        /// speed of cmd
        _speed: u8,

        #[argh(option)]
        /// direction
        _direction: Option<String>,
    }

    let actual = Cmd::redact_arg_values(&["program-name"], &["5"]).unwrap();
    assert_eq!(actual, &["program-name", "speed"]);
}

#[test]
fn redact_arg_values_subcommand() {
    #[derive(FromArgs, Debug)]
    /// Short description
    struct Cmd {
        #[argh(positional, arg_name = "speed")]
        /// speed of cmd
        _speed: u8,

        #[argh(subcommand)]
        /// means of transportation
        _means: MeansSubcommand,
    }

    #[derive(FromArgs, Debug)]
    /// Short description
    #[argh(subcommand)]
    enum MeansSubcommand {
        Walking(WalkingSubcommand),
        Biking(BikingSubcommand),
        Driving(DrivingSubcommand),
    }

    #[derive(FromArgs, Debug)]
    #[argh(subcommand, name = "walking")]
    /// Short description
    struct WalkingSubcommand {
        #[argh(option)]
        /// a song to listen to
        _music: String,
    }

    #[derive(FromArgs, Debug)]
    #[argh(subcommand, name = "biking")]
    /// Short description
    struct BikingSubcommand {}
    #[derive(FromArgs, Debug)]
    #[argh(subcommand, name = "driving")]
    /// short description
    struct DrivingSubcommand {}

    let actual =
        Cmd::redact_arg_values(&["program-name"], &["5", "walking", "--music", "Bach"]).unwrap();
    assert_eq!(actual, &["program-name", "speed", "walking", "--music"]);
}

#[test]
fn redact_arg_values_subcommand_with_space_in_name() {
    #[derive(FromArgs, Debug)]
    /// Short description
    struct Cmd {
        #[argh(positional, arg_name = "speed")]
        /// speed of cmd
        _speed: u8,

        #[argh(subcommand)]
        /// means of transportation
        _means: MeansSubcommand,
    }

    #[derive(FromArgs, Debug)]
    /// Short description
    #[argh(subcommand)]
    enum MeansSubcommand {
        Walking(WalkingSubcommand),
        Biking(BikingSubcommand),
    }

    #[derive(FromArgs, Debug)]
    #[argh(subcommand, name = "has space")]
    /// Short description
    struct WalkingSubcommand {
        #[argh(option)]
        /// a song to listen to
        _music: String,
    }

    #[derive(FromArgs, Debug)]
    #[argh(subcommand, name = "biking")]
    /// Short description
    struct BikingSubcommand {}

    let actual =
        Cmd::redact_arg_values(&["program-name"], &["5", "has space", "--music", "Bach"]).unwrap();
    assert_eq!(actual, &["program-name", "speed", "has space", "--music"]);
}

#[test]
fn redact_arg_values_produces_help() {
    #[derive(argh::FromArgs, Debug, PartialEq)]
    /// Woot
    struct Repeating {
        #[argh(option, short = 'n')]
        /// fooey
        n: Vec<String>,
    }

    assert_eq!(
        Repeating::redact_arg_values(&["program-name"], &["--help"]),
        Err(argh::EarlyExit {
            output: r###"Usage: program-name [-n <n...>]

Woot

Options:
  -n, --n           fooey
  --help, help      display usage information
"###
            .to_owned(),
            status: Ok(()),
        }),
    );
}

#[test]
fn redact_arg_values_produces_errors_with_bad_arguments() {
    #[derive(argh::FromArgs, Debug, PartialEq)]
    /// Woot
    struct Cmd {
        #[argh(option, short = 'n')]
        /// fooey
        n: String,
    }

    assert_eq!(
        Cmd::redact_arg_values(&["program-name"], &["--n"]),
        Err(argh::EarlyExit {
            output: "No value provided for option '--n'.\n".to_owned(),
            status: Err(()),
        }),
    );
}

#[test]
fn redact_arg_values_does_not_warn_if_used() {
    #[forbid(unused)]
    #[derive(FromArgs, Debug)]
    /// Short description
    struct Cmd {
        #[argh(positional)]
        /// speed of cmd
        speed: u8,
    }

    let cmd = Cmd::from_args(&["program-name"], &["5"]).unwrap();
    assert_eq!(cmd.speed, 5);

    let actual = Cmd::redact_arg_values(&["program-name"], &["5"]).unwrap();
    assert_eq!(actual, &["program-name", "speed"]);
}

#[test]
fn subcommand_does_not_panic() {
    #[derive(FromArgs, PartialEq, Debug)]
    #[argh(subcommand)]
    enum SubCommandEnum {
        Cmd(SubCommand),
    }

    #[derive(FromArgs, PartialEq, Debug)]
    /// First subcommand.
    #[argh(subcommand, name = "one")]
    struct SubCommand {
        #[argh(positional)]
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

    // Passing no subcommand name to an emum
    assert_eq!(
        SubCommandEnum::from_args(&[], &["5"]).unwrap_err(),
        argh::EarlyExit { output: "no subcommand name".into(), status: Err(()) },
    );

    assert_eq!(
        SubCommandEnum::redact_arg_values(&[], &["5"]).unwrap_err(),
        argh::EarlyExit { output: "no subcommand name".into(), status: Err(()) },
    );

    // Passing unknown subcommand name to an emum
    assert_eq!(
        SubCommandEnum::from_args(&["fooey"], &["5"]).unwrap_err(),
        argh::EarlyExit { output: "no subcommand matched".into(), status: Err(()) },
    );

    assert_eq!(
        SubCommandEnum::redact_arg_values(&["fooey"], &["5"]).unwrap_err(),
        argh::EarlyExit { output: "no subcommand matched".into(), status: Err(()) },
    );

    // Passing unknown subcommand name to a struct
    assert_eq!(
        SubCommand::redact_arg_values(&[], &["5"]).unwrap_err(),
        argh::EarlyExit { output: "no subcommand name".into(), status: Err(()) },
    );
}

#[test]
fn long_alphanumeric() {
    #[derive(FromArgs)]
    /// Short description
    struct Cmd {
        #[argh(option, long = "ac97")]
        /// fooey
        ac97: String,
    }

    let cmd = Cmd::from_args(&["cmdname"], &["--ac97", "bar"]).unwrap();
    assert_eq!(cmd.ac97, "bar");
}
