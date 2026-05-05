use clap::{ArgAction, Parser};

#[derive(Debug, Parser)]
#[command(
    name = "kora",
    version,
    about = "KORA Opinionated Remote Access",
    disable_help_flag = false,
    disable_version_flag = true,
)]
pub struct Cli {

    #[arg(short = 'v', long, action = ArgAction::SetTrue)]
    pub version: bool,

    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true,
        action = ArgAction::Append
    )]
    pub ssh_args: Vec<String>,
}