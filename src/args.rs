use clap::{ArgEnum, CommandFactory, ErrorKind, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Debug)]
pub enum ColorMode {
    Never,
    Auto,
    Always,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Debug)]
pub enum EscapeMode {
    No,
    Yes,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Debug)]
pub enum KeyOrder {
    Asc,
    Desc,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Use a debugger to dump environment variable string from memory of remote process
    RemoteEnvStringDump { pid: u32 },
}

/// Print environment variables
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// separate each output with NUL, not newline
    #[clap(short = '0', long)]
    pub null: bool,

    /// Controls colorfulness of output
    #[clap(long, arg_enum, required = false)]
    pub color: Option<ColorMode>,

    /// Display outputs in alphabetical order of names of environment variables
    #[clap(long, arg_enum, required = false)]
    pub key_order: Option<KeyOrder>,

    /// Escape control characters, for example line breaks
    #[clap(long, arg_enum, required = false)]
    pub escape: Option<EscapeMode>,

    /// Show the environment variables recorded by a file. It expects the same format as /proc/<fd>/environ file on Linux.
    #[clap(long, parse(from_os_str), required = false)]
    pub by_env_string: Option<PathBuf>,

    /// Specified variable name(s)
    #[clap(required = false)]
    pub variables: Vec<String>,

    #[clap(subcommand)]
    pub command: Option<Commands>,
}

pub fn parse() -> Args {
    let mut args = Args::parse();

    if let Some(Commands::RemoteEnvStringDump { .. }) = args.command {
        args.null = true;
        if args.color == Some(ColorMode::Auto) {
            args.color = Some(ColorMode::Never);
        }
    }

    if args.color == Some(ColorMode::Never) {
        colored::control::set_override(false);
    }

    if args.null && (args.color == Some(ColorMode::Always) || args.escape == Some(EscapeMode::Yes))
    {
        let mut cmd = Args::command();
        cmd.error(
            ErrorKind::ArgumentConflict,
            "Null mode cannot be used together with rich format output, such as color mode or escape mode",
        ).exit();
    }

    if !args.variables.is_empty() && args.key_order.is_some() {
        let mut cmd = Args::command();
        cmd.error(
            ErrorKind::ArgumentConflict,
            "Providing VARIABLES does not work with key-order mode",
        )
        .exit();
    }

    args
}
