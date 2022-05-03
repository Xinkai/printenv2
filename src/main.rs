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

use args::Commands;
use definition::AppResult;
use env::{parse_env_var_string, sort_pairs, RecordPair};
use printer::Printer;

fn main() -> AppResult<()> {
    let args = args::parse();

    match args.command {
        Some(Commands::RemoteEnvStringDump {}) => {
            println!(
                r##"#!/bin/sh

set -eu

OUTPUT=$(mktemp --quiet)

cat << EOF | gdb --pid=$1
set pagination off
set variable \$env = (char**) __environ
set variable \$i=0
while (\$env[\$i] != 0)
  set variable \$pos=0
  set variable \$char=1
  while (\$char != 0)
    set variable \$char=\$env[\$i][\$pos++]
    append binary value $OUTPUT \$char
  end
  set \$i = \$i+1
end
EOF

cat "$OUTPUT"
rm "$OUTPUT"
"##
            );

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
                None => std::env::vars_os()
                    .map(|(key, value)| {
                        (
                            platform_ext::os_string_to_u8_vec(&key),
                            platform_ext::os_string_to_u8_vec(&value),
                        )
                    })
                    .collect(),
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
