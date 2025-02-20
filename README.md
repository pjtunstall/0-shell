# 0-shell

## Table of Contents

- [What is this?](#what-is-this?)
- [Regarding the name](#regarding-the-name)
- [Deviations](#deviations)

## What is this?

A simple shell written in Rust. `0-shell` is one of the 01-Edu projects. According to [the brief](https://github.com/01-edu/public/tree/master/subjects/0-shell), it should implement at least the following ten commands, four built-in and six external--see below regarding this distinction:

Built-in:

- echo
- exit
- cd
- pwd

External:

- ls, including the flags -l, -a and -F
- cat
- cp
- rm, including the flag -r
- mv
- mkdir

Also, `Ctrl + D` to exit the shell. We're told that these commands "need to be implemented from scratch, without calling any external binaries." Related, but somewhat unclear to me is the following paragraph:

> Through the 0-shell you will get to the core of the Unix system and explore an important part of this system’s API which is the process creation and synchronization. Executing a command inside a shell implies creating a new process, which execution and final state will be monitored by its parents processes. This set of functions will be the key to success for your project.

A normal shell would itself handle `cd`, `exit`, `pwd`, `echo` (built-in commands), but call external binaries for `ls`, `cat`, `cp`, `rm`, `mv`, `mkdir` (external commands). (To check this for a given shell, you can enter `type <command>`.) I'm guessing this paragraph is a relic of the task as it was originally conceived, before commit 9e308f2: "fix(0shell): remove mandatory use of low-level system calls and make it bonus". An alternative possibility is that the authors intended to make a distinction between internal and external binaries, and have us spawn a new process for any of the commands that I've labeled external.

## Regarding the name

Rust's build tool, Cargo, doesn't allow a package name to begin with a numeral, hence the package is called `zero-shell`. For all other purposes, the name is `0-shell`, as required by the brief. When you build the project, either with `cargo run` (to build and run in one step) or `cargo build` (to build only) or `cargo build --release` (to build in release mode), the binary will be named `0-shell`.

## Deviations

The 01Edu instructions say, "This interpreter must display at least a simple `$`". To make my shell more distinctive, I chose to take `$` in a generic sense. I claim my "simple `$`" looks like this: `▶`. Or, to put it another way, yes, my program displays "at least a simple `$`"--indeed, it something does better than that: it displays a `▶`.
