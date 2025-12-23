#![cfg(feature = "help")]

use argh::FromArgs;

#[test]
fn test_subcommand_synonyms() {
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
    }

    #[derive(FromArgs, PartialEq, Debug)]
    /// First subcommand.
    #[argh(subcommand, name = "one", synonyms = ["uno", "eins"])]
    struct SubCommandOne {
        #[argh(option)]
        /// how many x
        x: usize,
    }

    let one = TopLevel::from_args(&["cmdname"], &["one", "--x", "2"]).expect("sc 1");
    assert_eq!(one, TopLevel { nested: MySubCommandEnum::One(SubCommandOne { x: 2 }) });

    let uno = TopLevel::from_args(&["cmdname"], &["uno", "--x", "2"]).expect("sc uno");
    assert_eq!(uno, TopLevel { nested: MySubCommandEnum::One(SubCommandOne { x: 2 }) });

    let eins = TopLevel::from_args(&["cmdname"], &["eins", "--x", "2"]).expect("sc eins");
    assert_eq!(eins, TopLevel { nested: MySubCommandEnum::One(SubCommandOne { x: 2 }) });
}

#[test]
fn test_option_synonyms() {
    #[derive(FromArgs, PartialEq, Debug)]
    /// Command with option synonyms.
    struct Cmd {
        #[argh(option, long = "foo", synonyms = ["bar", "baz"])]
        /// foo option
        foo: String,
    }

    let a = Cmd::from_args(&["cmd"], &["--foo", "value"]).unwrap();
    assert_eq!(a.foo, "value");

    let b = Cmd::from_args(&["cmd"], &["--bar", "value"]).unwrap();
    assert_eq!(b.foo, "value");

    let c = Cmd::from_args(&["cmd"], &["--baz", "value"]).unwrap();
    assert_eq!(c.foo, "value");
}

#[test]
fn test_switch_synonyms() {
    #[derive(FromArgs, PartialEq, Debug)]
    /// Command with switch synonyms.
    struct Cmd {
        #[argh(switch, long = "foo", synonyms = ["bar", "baz"])]
        /// foo switch
        foo: bool,
    }

    let a = Cmd::from_args(&["cmd"], &["--foo"]).unwrap();
    assert!(a.foo);

    let b = Cmd::from_args(&["cmd"], &["--bar"]).unwrap();
    assert!(b.foo);

    let c = Cmd::from_args(&["cmd"], &["--baz"]).unwrap();
    assert!(c.foo);
}
