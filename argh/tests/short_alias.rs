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
// No short alias for this one
struct SubCommandTwo {
    #[argh(switch)]
    /// bar
    bar: bool,
}

#[test]
fn test_short_alias_dispatch() {
    let expected = TopLevel { nested: MySubCommandEnum::One(SubCommandOne { fooey: true }) };

    // Test with full name "one"
    let actual = TopLevel::from_args(&["cmd"], &["one", "--fooey"]).expect("failed parsing 'one'");
    assert_eq!(actual, expected);

    // Test with short alias "o"
    let actual_short =
        TopLevel::from_args(&["cmd"], &["o", "--fooey"]).expect("failed parsing 'o'");
    assert_eq!(actual_short, expected);
}

#[test]
fn test_short_alias_redaction() {
    // Verify that redaction also works with short aliases
    let args = vec!["o", "--fooey"];
    let redacted = TopLevel::redact_arg_values(&["cmd"], &args).expect("redaction failed");
    // Since it's a switch, it might be kept or redacted depending on impl, but we check matching.
    // Redaction usually returns the args with values redacted. For switch there are no values.
    // We mainly want to ensure it doesn't error with "no subcommand matched".
    assert!(!redacted.is_empty());
}

#[test]
fn test_no_short_alias_for_two() {
    // "two" has no short alias, so "t" should fail (unless "t" is prefix matching? argh doesn't do prefix matching for subcommands by default I think, but let's see)
    // Actually argh requires exact match for subcommands unless strict is disabled?
    // Let's assume strict.

    let res = TopLevel::from_args(&["cmd"], &["t", "--bar"]);
    assert!(res.is_err());
}
