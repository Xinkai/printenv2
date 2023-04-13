#![deny(warnings)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Stdout, Write};

mod args;
mod definition;
mod env;
mod platform_ext;
mod printer;
#[cfg(unix_apple_sysctl)]
mod remote_apple_sysctl;
#[cfg(debugger_helper)]
mod remote_debugger_helper;
#[cfg(all(remote_env, target_os = "linux"))]
mod remote_linux_procfs;
#[cfg(unix_kvm)]
mod remote_unix_kvm;
#[cfg(all(remote_env, target_family = "windows"))]
mod remote_windows;

use definition::AppResult;
use printer::Printer;

fn main() -> AppResult<()> {
    let args = args::parse();

    #[cfg(debugger_helper)]
    if args.debugger_helper == Some(args::DebuggerHelper::Gdb) {
        colored::control::set_override(false);
        println!("{}", remote_debugger_helper::get_gdb_helper());
        return Ok(());
    }

    #[cfg(remote_env)]
    let pid = args.pid;

    #[cfg(not(remote_env))]
    let pid: Option<u32> = None;

    let env = {
        let mut env = match (args.load, pid) {
            (Some(path), None) => {
                let mut content = Vec::new();
                if path == OsString::from("-") {
                    let stdin = std::io::stdin();
                    let mut reader = stdin.lock();
                    reader.read_to_end(&mut content)?;
                } else {
                    let mut file = File::open(path)?;
                    file.read_to_end(&mut content)?;
                }
                env::Env::from(content)
            }
            #[cfg(remote_env)]
            (None, Some(pid)) => env::Env::from(env::remote::get_environment_string(pid)?),
            (None, None) => env::Env::new(),
            _ => unreachable!(),
        };

        if !args.variables.is_empty() {
            env.filter_keys(&args.variables);
        }

        env.sort_by_key(args.key_order);
        env
    };

    let mut printer = Printer::default();

    // Override default printer behaviors
    if args.null {
        printer.null = args.null;
    }

    if args.json {
        printer.json = args.json;
    }

    if !args.variables.is_empty() {
        printer.include_keys = false;
    }

    printer.color = args.color;
    if let Some(escape) = args.escape {
        printer.escape = escape;
    }

    let output = printer.print(&env)?;
    Stdout::write(&mut std::io::stdout(), &output)?;

    if output.is_empty() {
        std::process::exit(1);
    }
    Ok(())
}
