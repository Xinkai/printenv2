#![deny(warnings)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(clippy::cargo)]
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;

use std::process::exit;

mod args;
mod definition;
mod env;
mod platform_ext;
mod printer;
#[cfg(target_family = "unix")]
mod remote_unix;
#[cfg(target_family = "windows")]
mod remote_windows;

use args::Commands;
use definition::AppResult;
use env::{parse_env_var_string, sort_pairs, RecordPair};
use printer::Printer;

fn main() -> AppResult<()> {
    let args = args::parse();

    match args.command {
        Some(Commands::RemoteEnvStringDump { pid }) => {
            #[cfg(target_family = "unix")]
            let output = { remote_unix::get_gdb_helper(pid) };

            #[cfg(target_family = "windows")]
            let output = { remote_windows::get_environment_string(pid)? };

            colored::control::set_override(false);
            print!("{}", output);

            Ok(())
        }
        None => {
            let mut printer = Printer::default();

            // Override default printer behaviors
            if args.null {
                printer.null = args.null;
            }

            if !args.variables.is_empty() {
                printer.variables = Some(&args.variables);
            }

            if let Some(color) = args.color {
                printer.color = color;
            }
            if let Some(escape) = args.escape {
                printer.escape = escape;
            }

            let mut results: Vec<RecordPair> = match args.by_env_string {
                Some(path) => {
                    let mut content = Vec::new();
                    if path == OsString::from("-") {
                        let stdin = std::io::stdin();
                        let mut reader = stdin.lock();
                        reader.read_to_end(&mut content)?;
                    } else {
                        let mut file = File::open(path)?;
                        file.read_to_end(&mut content)?;
                    }
                    parse_env_var_string(&content)
                }
                None => env::get_record_pairs_for_current_process(),
            };

            if let Some(key_order) = args.key_order {
                sort_pairs(key_order, &mut results);
            }

            match printer.print(&results) {
                Some(_) => Ok(()),
                None => exit(1),
            }
        }
    }
}
