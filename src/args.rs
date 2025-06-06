use clap::{CommandFactory, Parser, ValueEnum, error::ErrorKind};
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
    None,
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
    /// Use NUL as delimiter instead of newline
    #[clap(short = '0', long)]
    pub null: bool,

    /// Read environment variables of another running process
    #[cfg(remote_env)]
    #[clap(long, required = false)]
    pub pid: Option<u32>,

    #[cfg(debugger_helper)]
    #[clap(long, value_enum, required = false)]
    /// Print out a script to invoke a debugger for inspecting the in-present environment variables of another process
    pub debugger_helper: Option<DebuggerHelper>,

    /// Control colorfulness of output
    #[clap(long, value_enum, required = false)]
    #[arg(default_value_t = ColorMode::Auto)]
    pub color: ColorMode,

    /// Display outputs in alphabetical order of names of environment variables
    #[clap(long, value_enum, required = false)]
    #[arg(default_value_t = KeyOrder::None)]
    pub key_order: KeyOrder,

    /// Escape control characters, for example line breaks [default: yes unless --null or --json is used]
    #[clap(long, value_enum, required = false)]
    pub escape: Option<EscapeMode>,

    /// Load environment variables from a file. The file should be in the same format as the output format of --null
    #[clap(long, value_parser = clap::value_parser!(PathBuf), required = false)]
    pub load: Option<PathBuf>,

    /// Output in JSON format
    #[clap(long)]
    pub json: bool,

    /// Filter by environment variable names, also omit key names
    #[clap(required = false)]
    pub variables: Vec<String>,
}

pub fn parse() -> Args {
    let args = Args::parse();

    if args.color == ColorMode::Never || (args.null && args.color == ColorMode::Auto) {
        colored::control::set_override(false);
    }

    if args.null && (args.color == ColorMode::Always || args.escape == Some(EscapeMode::Yes)) {
        let mut cmd = Args::command();
        cmd.error(
            ErrorKind::ArgumentConflict,
            "Null mode cannot be used together with other rich-format switches",
        )
        .exit();
    }

    if !args.variables.is_empty() && args.key_order != KeyOrder::None {
        let mut cmd = Args::command();
        cmd.error(ErrorKind::ArgumentConflict, "VARIABLES cannot be sorted")
            .exit();
    }

    #[cfg(debugger_helper)]
    #[allow(clippy::collapsible_if)]
    if args.debugger_helper.is_some() {
        if args.null
            || (args.color == ColorMode::Always || args.escape.is_some())
            || !args.variables.is_empty()
            || args.key_order != KeyOrder::None
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

        if args.color == ColorMode::Always || args.escape == Some(EscapeMode::Yes) {
            let mut cmd = Args::command();
            cmd.error(
                ErrorKind::ArgumentConflict,
                "JSON mode cannot be used together with other rich-format switches",
            )
            .exit();
        }
    }

    args
}
