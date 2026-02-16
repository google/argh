use argh::FromArgs;

#[derive(FromArgs, Debug)]
#[argh(
    description = "{command_name} is a tool to reach new heights.\n\n\
    Start exploring new heights:\n\n\
    \u{00A0} {command_name} --height 5 jump\n\
    ",
    example = "\
    {command_name} --height 5\n\
    {command_name} --height 5 j\n\
    {command_name} --height 5 --pilot-nickname Wes jump"
)]
pub struct CliArgs {
    /// how high to go
    #[argh(option)]
    height: usize,
    /// an optional nickname for the pilot
    #[argh(option)]
    pilot_nickname: Option<String>,
    /// command to execute
    #[argh(subcommand)]
    pub command: Command,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
pub enum Command {
    Jump(JumpCmd),
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "jump", short = 'j')]
/// whether or not to jump
pub struct JumpCmd {}

fn main() {
    let args: CliArgs = argh::from_env();
    println!("{:#?}", args);
}
