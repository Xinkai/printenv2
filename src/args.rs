use clap::{error::ErrorKind, CommandFactory, Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum ColorMode {
    Never,
    Auto,
    Always,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum EscapeMode {
    No,
    Yes,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum KeyOrder {
    Asc,
    Desc,
}

#[cfg(debugger_helper)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum DebuggerHelper {
    Gdb,
}

/// Print environment variables
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Separate each output with NUL, not newline
    #[clap(short = '0', long)]
    pub null: bool,

    /// Use environment variables of another running process.
    #[cfg(remote_env)]
    #[clap(long, required = false)]
    pub pid: Option<u32>,

    #[cfg(debugger_helper)]
    #[clap(long, value_enum, required = false)]
    /// Dumps a debugging script for inspecting the in-present environment variables of another process.
    pub debugger_helper: Option<DebuggerHelper>,

    /// Controls colorfulness of output
    #[clap(long, value_enum, required = false)]
    pub color: Option<ColorMode>,

    /// Display outputs in alphabetical order of names of environment variables
    #[clap(long, value_enum, required = false)]
    pub key_order: Option<KeyOrder>,

    /// Escape control characters, for example line breaks
    #[clap(long, value_enum, required = false)]
    pub escape: Option<EscapeMode>,

    /// Load environment variables from a file. It expects the same format as /proc/<fd>/environ file on Linux.
    #[clap(long, value_parser = clap::value_parser!(PathBuf), required = false)]
    pub load: Option<PathBuf>,

    /// Output in JSON format
    #[clap(long)]
    pub json: bool,

    /// Specified variable name(s)
    #[clap(required = false)]
    pub variables: Vec<String>,
}

pub fn parse() -> Args {
    let args = Args::parse();

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

    #[cfg(debugger_helper)]
    #[allow(clippy::collapsible_if)]
    if args.debugger_helper.is_some() {
        if args.null
            || (args.color == Some(ColorMode::Always) || args.escape == Some(EscapeMode::Yes))
            || !args.variables.is_empty()
            || args.key_order.is_some()
            || args.json
        {
            let mut cmd = Args::command();
            cmd.error(
                ErrorKind::ArgumentConflict,
                "--debugger-helper does not work with other arguments",
            )
            .exit();
        }
    }

    #[cfg(remote_env)]
    if args.pid.is_some() && args.load.is_some() {
        let mut cmd = Args::command();
        cmd.error(
            ErrorKind::ArgumentConflict,
            "--pid and --load cannot be used together",
        )
        .exit();
    }

    if args.null && args.load.is_some() {
        let mut cmd = Args::command();
        cmd.error(
            ErrorKind::ArgumentConflict,
            "--null and --load cannot be used together",
        )
        .exit();
    }

    if args.json {
        if args.null {
            let mut cmd = Args::command();
            cmd.error(
                ErrorKind::ArgumentConflict,
                "--null and --json cannot be used together",
            )
            .exit();
        }

        if args.color == Some(ColorMode::Always) || args.escape == Some(EscapeMode::Yes) {
            let mut cmd = Args::command();
            cmd.error(
                ErrorKind::ArgumentConflict,
                "JSON mode cannot be used together with rich format output, such as color mode or escape mode",
            )
            .exit();
        }
    }

    args
}
