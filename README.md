# 0-shell

## Table of Contents

- [What is this?](#what-is-this?)
- [Audit](#audit)
  - [Prompt](#prompt)
  - [echo something](#echo-something)
  - [Last orders, please](#last-orders-please)
- [Regarding the name](#regarding-the-name)

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

Also, `Ctrl + D` to exit the shell. We're told that these commands "need to be implemented from scratch, without calling any external binaries."

Related to the last point, but somewhat unclear to me is the following paragraph:

> Through the 0-shell you will get to the core of the Unix system and explore an important part of this system’s API which is the process creation and synchronization. Executing a command inside a shell implies creating a new process, which execution and final state will be monitored by its parents processes. This set of functions will be the key to success for your project.

A normal shell would itself handle `cd`, `exit`, `pwd`, `echo` (built-in commands), but call external binaries for `ls`, `cat`, `cp`, `rm`, `mv`, `mkdir` (external commands). (To check this for a given shell, you can enter `type <command>`.) I'm guessing this paragraph is a relic of the task as it was originally conceived, before commit 9e308f2: "fix(0shell): remove mandatory use of low-level system calls and make it bonus". An alternative possibility is that the authors intended to make a distinction between internal and external binaries, and have us spawn a new process for any of the commands that I've labeled external. But the advice to take [BusyBox](https://en.wikipedia.org/wiki/BusyBox) as an example points towards 0-shell being a single executable.

## Audit

### Prompt

The 01Edu instructions say, "This interpreter must display at least a simple `$`". To make my shell more distinctive, I chose to take `$` in a generic sense; I claim my "simple `$`" looks like this: `▶`. Or, to put it another way, my program displays "at least a simple `$`"--in fact, it something does better than that: it displays a `▶`.

### echo something

One of the audit instructions is:

**Try to run the command "echo "something!"". Do the same in your computer terminal.**

It then asks, "Can you confirm that the displayed message of the project is exactly the same as the computer terminal?" I've made the following assumptions about the text to be entered, besides the obvious one that "your computer terminal" is not also running 0-shell!

- The outer quotes are to be omitted, as in the instruction for the next item.
- The text inside those outer quotes is to be entered unchanged in shells which don't use `!` as a special character for history expension, such as PowerShell and fish.
- In shells with default history expension (such as Bash, Zsh, csh, and tcsh), the `!` is to be escaped with a preceding `\`. Otherwise, Bash and Zsh will display `dquote>` in response to any input till you close the inner quotes.
- In POSIX shell (sh), dash, and ksh, that have optional histotory expansion, the text should be entered depending on which option is currently selected. I gather the default is no history expansion with `!`.
- It's my understanding that BusyBox's default shell (ash) does not treat `!` as special (it lacks history expansion by default, similar to dash). However, if built with hush (another shell included in BusyBox), history expansion with `!` can be optionally enabled.

Since there is no requirement to implement special behavior for `!` or guidance on which shell to use as a standard (unless we can take the mention on BusyBox as a hint), I take this to be an oversight.

### Last orders, please

See `integration.rs` for an integration test that covers the last section of the audit (before the bonus questions). I've assumed that the omission of `txt` extension from `new_doc.txt` on the third mention is accidental.

## Regarding the name

Rust's build tool, Cargo, doesn't allow a package name to begin with a numeral, hence the package is called `zero-shell`. For all other purposes, the name is `0-shell`, as required by the brief. When you build the project, either with `cargo run` (to build and run in one step) or `cargo build` (to build only) or `cargo build --release` (to build in release mode), the binary will be named `0-shell`.

## Testing

Tests are best run in single-threaded mode ...

```zsh
cargo test -- --test-threads=1
```

... rather than in the default parallel mode, `cargo test`. This is necessary to prevent tests that change the current directory from interfering with tests that check the identity of the current ditectory, or the existence or nonexistence or contents of files they've created in it. The success test for `pwd`, for example, expects the current directory to be `0-shell`, but the `cd` test temporarily changes the current directory. The integration test in `integration.rs` also changes the current directory briefly.

## Further

See [todo.md](todo.md) for possible further developments and topics to explore.
