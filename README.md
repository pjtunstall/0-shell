# 0-shell

## Table of Contents

- [What is this?](#what-is-this?)
- [Job control](#job-control)
- [Audit](#audit)
  - [Name](#name)
  - [echo something](#echo-something)
  - [Last orders, please](#last-orders-please)
  - [Deviations](#deviations)
- [Testing](#testing)
- [Further](#further)
- [Notes](#notes)

## What is this?

This is my take on the [01-Founders/01-Edu project of the same
name](https://github.com/01-edu/public/tree/master/subjects/0-shell) (commit
b0b9f3d). The object of the exercise is to learn about shells by implementing
some core Unix commands from scratch without using external binaries or existing shell
utilities.<sup id="ref-f1">[1](#f1)</sup>

We're required to recreate at least the following ten commands:

Built-in:<sup id="ref-f2">[2](#f2)</sup>

- `echo`
- `exit`
- `cd`
- `pwd`

External:

- `ls`, including the flags `-l`, `-a` and `-F`
- `cat`
- `cp`
- `rm`, including the flag `-r`
- `mv`
- `mkdir`

Also, Ctrl+D to exit our shell.

We're told:

> Through the 0-shell you will get to the core of the Unix system and explore an
> important part of this system’s API which is the process creation and
> synchronization. Executing a command inside a shell implies creating a new
> process, which execution and final state will be monitored by its parents
> processes. This set of functions will be the key to success for your project.

For the external commands above, I following the hint to take
[BusyBox](https://en.wikipedia.org/wiki/BusyBox) as an example. When such a
command is entered, the shell forks. Its child then execs itself with the
command as an argument. On finding itself launched with such an argument, the
program calls the relevant function.<sup id="ref-f3">[3](#f3)</sup>

I've added several bonus features, including:

- extra commands:
  - `man`
  - `sleep`
  - `touch`
- colors
- auto-completion
- command history
- redirection
- environment variables

## Job control

I've also done the optional extra project
[job-control](https://github.com/01-edu/public/tree/master/subjects/0-shell/job-control),
which extends 0-shell with these features:

- Ctrl+C: terminate a child process and return to 0-shell
- Ctrl+Z: pause a child process and return to 0-shell
- `jobs`: list background jobs (aka pipelines, aka process groups)
- `fg`: restart a paused job in the foreground
- `bg`: restart one or more jobs in the background
- `kill`: terminate a job

While the instructions tell us that our program should respect the same
principles as 0-shell, one of the job-control [audit
questions](https://github.com/01-edu/public/blob/master/subjects/0-shell/job-control/audit.md)
implies that it should now launch external binaries, at least for commands
included in the core project.<sup id="ref-f5">[5](#f5)</sup>

To accomplish this, I've kept my custom versions of the listed externals,
and, for other externals, I fork the process and let the child exec itself
with the given command as an argument.

I've also implemented the features not mentioned in the instructions but implied
by the example, namely:

- additional flags for `ls`:
  - `-r`: reverse
  - `-R`: recurse
- redirection from file descriptor to file: `2>1`<sup id="ref-f5">[5](#f5)</sup>

## Setup

Make sure you have [Rust](https://www.rust-lang.org/tools/install) installed.
Then

```sh
git clone https://github.com/pjtunstall/0-shell
cd 0-shell
cargo run
```

## Audit

### Name

I've named the internal package zero-shell because Rust's package/build
manager Cargo disallows package names starting with numerals. Howevere, ensured
that the binary itself is called `0-shell`.

- Library: `zero_shell`
- Binary output: `target/debug/0-shell`

### echo something

Depending on your shell and how it's configured, you may need to escape the
bang in `echo "something!"` when you type it in your own shell, thus `\!`.

### Last orders, please

See `integration.rs` for an integration test that covers the last section of the
audit (before the bonus questions). I've assumed that the omission of `txt`
extension from `new_doc.txt` on the third mention is accidental.

### Deviations

There are many trivial deviations from Zsh (my default shell at the time when I
made my orignal version of the core project), even among the few items that I've
implemented: bold text, different prompts, use of red for error messages, ...
Lack of `!` for history expansion is discussed above. In Zsh, if you try to
`cat` to a directory and a file, the operation fails completely and doesn't
concatenated to the file. In my shell, it does what it can and reports any
failures. That's more consistent with how Zsh behaves with `rm`, say.

I've not been meticulous in mimicking the wording, word order, or capitalization
of error messages. In some cases, I've aimed for greater consistency than I
found in Zsh. Thus, Zsh and Bash both have `cat: tests: Is a directory` when one
of the sources is a directory, while I have `cat: Is a directory: tests`. But I
also capitalize when one of the targets is a directory--`0-shell: Is a
directory: tests`--unlike Zsh: `zsh: is a directory: tests`. In this case, Bash
follows yet another pattern: `bash: line 1: tests: Is a directory`.

You'll likely find other examples.

## Testing

Tests should be run in single-threaded mode ...

```sh
cargo test -- --test-threads=1
```

... rather than in the default parallel mode, `cargo test`. This is to
prevent tests that change the current directory from interfering with tests that
check the the current ditectory or its contents.

## Further

See [todo.md](todo.md) for possible further developments and topics to explore.

## Notes

<a id="f1" href="#ref-f1">1</a>: I've written my code for a Unix-like OS. This
assumption is implicit in my handling of process groups and signals, and
associated use of `libc`, for example. An earlier version (from before I
implemented job control) did aim to be platform-agnostic, hence the Windows
variants for obtaining fs metadata the `ls::system` module.[↩](#ref-f1)

<a id="f2" href="#ref-f2">2</a>: A traditional Unix shell, such as Bash, treats
certain commands as built-in utilities: `cd`, `exit`, `pwd`, `echo` (the first
two of necessity built-in). Other commands launch external binaries: `ls`,
`cat`, `cp`, `rm`, `mv`, `mkdir`. To check whether a command is a builtin for a
given shell, you can enter `type <command>`.[↩](#ref-f2)

<a id="f3" href="#ref-f3">3</a>: On installation, I gather that Busybox makes,
for example, `/bin/ls` a symbolic link pointing to `/bin/0_shell`, allowing it
act in place of a default shell. I haven't gone this far.[↩](#ref-f3)

<a id="f4" href="#ref-f4">4</a>: "then run `python &"`. I assume they didn't
want us write our own Python.[↩](#ref-f4)

<a id="f5" href="#ref-f5">5</a>: It's been suggested that this is a typo for
`1>&2` (duplicating a file descriptor), which my 0-shell also
handles.[↩](#ref-f5)
