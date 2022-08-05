printenv2
=========

A `printenv` rewrite in Rust.

Advantages over classic implementations
---------------------------------------
* Rich-format output
  * Colored mode
  * Sort mode: output can be sorted by environment variable names
  * Escape mode: 
    * Single-line: escape line break characters
    * Unprintable characters
* Remote mode. See notes
* Cross-platform

Installation
------------
* via package managers:
  * Arch Linux: `paru -S printenv2`
* via Cargo: Run `cargo install printenv2` if you already have Rust development environment setup.

Notes on Remote Mode
--------------------

`printenv2` comes with the ability to read environment variables of another running process.

Basic usage: 
```sh
# Make sure you have privilege to inspect the target process.
printenv2 --pid 1000
```

Platform-specifics:

| Platform    | Environment variables at startup | Environment variables in present                                                                                                                                                                     |
|-------------|----------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Linux       | `printenv2 --pid <PID>`          | Unsafe[^1].<br/>`printenv2 --debugger-helper` generates a shell script for that using `gdb`.<br/>`sh <(printenv2 --debugger-helper=gdb) <PID> \| printenv2 --load -`.<br/>`sudo` is likely required. |
| Windows     | Unsupported.                     | Unsafe[^1].<br/>`printenv2 --pid <PID>`                                                                                                                                                              |
| Unix (*BSD) | `printenv2 --pid <PID>`          | Unsafe[^1].<br/>`printenv2 --debugger-helper` generates a shell script for that using `gdb`.<br/>`sh <(printenv2 --debugger-helper=gdb) <PID> \| printenv2 --load -`.<br/>`sudo` is likely required. |
| macOS       | `printenv2 --pid <PID>`          | Unsupported.                                                                                                                                                                                         |
| Other       | Unsupported.                     | Unsupported.                                                                                                                                                                                         |

[^1]: Be careful. These methods use a debugger or undocumented APIs.

TODO
----
- [ ] Json output

License
-------

MIT
