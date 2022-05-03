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
* via Cargo: Run `cargo install printenv2` if you already have Rust development environment setup.

Notes on remote mode
--------------------

`printenv2` comes with the ability to read environment variables of another running process. However, mileage varies depending on the operating system. 

The following table shows how each platform is supported.

| Platform          | Environment variables at startup                                                         | Environment variables in present                                                                                                                                                                                                                                                                          |
|-------------------|------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Linux             | `printenv2 --by-env-string /proc/<PID>/environ`.<br/>`sudo` for processes you don't own. | Use at your risk.<br/>A debugger must be used to dump the memory where environment variables are stored. `printenv2 remote-env-dump` generates a shell script for that using `gdb`.<br/>`sh <(printenv2 remote-env-string-dump) <PID> \| printenv2 --by-env-string -`.<br/>`sudo` is likely required. |
| Other             | Unsupported.                                                                             | Unsupported.                                                                                                                                                                                                                                                                                              |

TODO
----
- [ ] Remote mode on more OSes

License
-------

MIT
