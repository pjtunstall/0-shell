# 0-shell

## What is this?

A simple shell written in Rust. `0-shell` is one of the 01-Edu projects. According to [the brief](https://github.com/01-edu/public/tree/master/subjects/0-shell), it should implement the following ten commands:

- echo
- cd
- ls, including the flags -l, -a and -F
- pwd
- cat
- cp
- rm, including the flag -r
- mv
- mkdir
- exit

It should also handle program interrupt with `Ctrl + D`.

## Regarding the name

Rust's build tool, Cargo, doesn't allow a package name to begin with a numeral, hence the package is called `zero-shell`. For all other purposes, the name is `0-shell`, as required by the brief. When you build the project, either with `cargo run` (to build and run in one step) or `cargo build` (to build only) or `cargo build --release` (to build in release mode), the binary will be named `0-shell`.
