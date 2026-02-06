use argh::FromArgs;

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
#[argh(subcommand, name = "one", short = 'o')]
struct SubCommandOne {
    #[argh(switch)]
    /// fooey
    fooey: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Second subcommand.
#[argh(subcommand, name = "two")]
struct SubCommandTwo {
    #[argh(switch)]
    /// bar
    bar: bool,
}

#[test]
fn test_subcommand_short_help() {
    let early_exit = TopLevel::from_args(&["cmd"], &["help"]).unwrap_err();
    let output = early_exit.output;

    // Check that 'one' has its short name 'o'
    assert!(output.contains("one  o"), "Help output should contain 'one  o'");
    // Check that 'two' does NOT have a short name (it should just be 'two')
    assert!(output.contains("two "), "Help output should contain 'two '");
    assert!(!output.contains("two  t"), "Help output should NOT contain 'two  t'");

    let expected_part = "  one  o            First subcommand.
  two               Second subcommand.";
    assert!(
        output.contains(expected_part),
        "Help output did not match expected subcommand list formatting. Got:
{}",
        output
    );
}

#[test]
fn test_subcommand_short_help_own() {
    // Invoke via full name
    let early_exit_full = TopLevel::from_args(&["cmd"], &["one", "help"]).unwrap_err();
    assert!(
        early_exit_full.output.contains("Usage: cmd one [--fooey]"),
        "Usage should be 'cmd one ...'"
    );

    // Invoke via short name
    let early_exit_short = TopLevel::from_args(&["cmd"], &["o", "help"]).unwrap_err();
    assert!(
        early_exit_short.output.contains("Usage: cmd one [--fooey]"),
        "Usage should still be 'cmd one ...' even if 'o' was used"
    );

    assert_eq!(early_exit_full.output, early_exit_short.output);
}
