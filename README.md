# 0-shell

## Table of Contents

- [What is this?](#what-is-this?)
- [Audit](#audit)
  - [Prompt](#prompt)
  - [echo something](#echo-something)
  - [Last orders, please](#last-orders-please)
  - [Job control](#job-control)
- [Regarding the name](#regarding-the-name)
- [Testing](#testing)
- [Deviations](#deviations)
- [Further](#further)
- [Notes](#notes)

## What is this?

This is my take on the [01-Founders/01-Edu project of the same name](https://github.com/01-edu/public/tree/master/subjects/0-shell) (commit b0b9f3d). The object of the exercise is to learn about shells and job-control by mimicking essential Unix shell behaviors without using external binaries or existing shell utilities.

We're required to recreate at least the following ten commands:

Built-in:<sup id="ref-f1">[1](#f1)</sup>

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

Also, `Ctrl + D` to exit our 'shell'.

We're told:

> Through the 0-shell you will get to the core of the Unix system and explore an important part of this system’s API which is the process creation and synchronization. Executing a command inside a shell implies creating a new process, which execution and final state will be monitored by its parents processes. This set of functions will be the key to success for your project.

Following the hint to take [BusyBox](https://en.wikipedia.org/wiki/BusyBox) as an example, I decided to make my 0-shell a single executable and have the process fork itself with a suitable argument when the user enters a command and let the main process wait while the child calls the relevant function.<sup id="ref-f2">[2](#f2)</sup>

I've added several bonus features, including:

- extra commands:
  - man
  - sleep
  - touch
- color for error messages
- auto-completion
- command history
- redirection
- environment variables

I'm also implementing some features from the optional extra project [job-control](https://github.com/01-edu/public/tree/master/subjects/0-shell/job-control):

- Ctrl+C: terminate a child process and return to the 'shell'
- Ctrl+Z: pause a child process and return to the 'shell'
- jobs: list background processes
- fg: restart a paused child process in the foreground
- bg: restart one or more child processes in the background
- kill: terminate a process

As in Bash, the arguments to `fg` and `bg` are job numbers (1, 2, ...) while `kill` expects a system-wide PID (process ID), such as `1881893`; the convention can be reversed, for any of these commands, by prefixing the number with `%`, thus: `kill %1` or `fg %1881893`.

For the remaining tasks on this optional, see [Audit: Job control](#job-control).

## Audit

### Prompt

The 01Edu instructions say, "This interpreter must display at least a simple `$`". I chose to take `$` in a generic sense; I claim my "simple `$`" looks like this: `▶`. Or, to put it another way, my program displays "at least a simple `$`"--in fact, it something does better than that: it displays a `▶`.

### echo something

One of the audit instructions is:

**Try to run the command "echo "something!"". Do the same in your computer terminal.**

It then asks, "Can you confirm that the displayed message of the project is exactly the same as the computer terminal?" I've made the following assumptions about the text to be entered, besides the obvious one that "your computer terminal" is not also running 0-shell! (Basically, your milage may vary, depending on your shell and how it's configured.)

- The outer quotes are to be omitted, as in the instruction for the next item.
- The text inside those outer quotes is to be entered unchanged in shells which don't use `!` as a special character for history expension, such as PowerShell and fish.
- In shells with default history expension (such as Zsh, csh, and tcsh), the `!` is to be escaped with a preceding `\`. Otherwise, these shells will display `dquote>` in response to any input till you close the inner quotes. In Bash, history expansion is disabled in non-interactive mode (e.g. `bash -c 'echo "something!"'`), so the bang works unescaped; in interactive Bash it only needs escaping if `histexpand` is enabled (the usual default), otherwise `echo "something!"` works as-is.
  - It works as is with the default settings for Bash in my current version of VS Code, for example.
- In POSIX shell (sh), dash, and ksh, that have optional history expansion, the text should be entered depending on which option is currently selected. I gather the default is no history expansion with `!`.
- It's my understanding that BusyBox's default shell (ash) does not treat `!` as special (it lacks history expansion by default, similar to dash). However, if built with hush (another shell included in BusyBox), history expansion with `!` can be optionally enabled.

Since there's no requirement to implement special behavior for `!` or guidance on which shell to use as a standard (unless we can take the mention on BusyBox as a hint), I consider this is an oversight.

### Last orders, please

See `integration.rs` for an integration test that covers the last section of the audit (before the bonus questions). I've assumed that the omission of `txt` extension from `new_doc.txt` on the third mention is accidental.

### Job control

This 0-shell meets the stated requirements for the optional extension project job-control, although not yet the following extra features implied by the example:

- two additional flags for `ls`, namely `-r` (reverse) and `-R` (recursive)
- redirection between file descriptors: `2>&1` (assuming `2>1` is a typo)

I have implemented a feature not stated in the instructions but implied by the audit questions: execution of arbitrary external binaries (apart from the commands we had to re-implement). But to pass the test, I still need to mimic Bash's behavior when Python is launched from 0-shell as a background process.

## Regarding the name

The root directory and repo are named `0-shell`, as required by the brief. Unfortunately clashing conventions have resulted in almost the maximum conceivable variants! Rust's build tool, Cargo, doesn't allow a package name to begin with a numeral, hence the package is called `zero-shell` and the lib and bin crates `zero_shell` according to Rust convention. When you build the project, either with `cargo run` (to build and run in one step) or `cargo build` (to build only) or `cargo build --release` (to build in release mode), a build script will rename the binary `0_shell`. It insists on an underscore in place of the dash. This makes the name more broadly compatible across operating systems, so I've decided not to process it further (e.g. wrapping the call to Cargo in a shell script, or using a Cargo extension or a build system like `make`).

## Testing

Tests should be run in single-threaded mode ...

```sh
cargo test -- --test-threads=1
```

... rather than in the default parallel mode, `cargo test`. This is necessary to prevent tests that change the current directory from interfering with tests that check the identity of the current ditectory, or the existence or nonexistence or contents of files they've created in it. The success test for `pwd`, for example, expects the current directory to be `0-shell`, but the `cd` test temporarily changes the current directory. The integration test in `integration.rs` also changes the current directory briefly. If it was just one or two such clashes, they could be guarded with a mutex, but relying on a mutex would put the burdon on anyone adding a test to any of the modules to remember to use it. It seems more robust to just run them all sequentially.

## Deviations

There are many trivial deviations from Zsh (my default shell at the time when I made my orignal version of the core project), even among the few items that I've implemented: bold text, different prompts, use of red for error messages, ... Lack of `!` for history expansion is discussed above. In Zsh, if you try to `cat` to a directory and a file, the operation fails completely and doesn't concatenated to the file. In my shell, it does what it can and reports any failures. That's more consistent with how Zsh behaves with `rm`, say.

I've not been meticulous in mimicking the wording, word order, or capitalization of error messages. In some cases, I've aimed for greater consistency than I found in Zsh. Thus, Zsh and Bash both have `cat: tests: Is a directory` when one of the sources is a directory, while I have `cat: Is a directory: tests`. But I also capitalize when one of the targets is a directory--`0-shell: Is a directory: tests`--unlike Zsh: `zsh: is a directory: tests`. In this case, Bash follows yet another pattern: `bash: line 1: tests: Is a directory`.

You'll likely find other examples.

## Further

See [todo.md](todo.md) for possible further developments and topics to explore.

## Notes

<a id="f1" href="#ref-f1">1</a>: A traditional Unix shell, such as Bash, treats certain commands as built-in utilities: `cd`, `exit`, `pwd`, `echo` (the first two of necessity built-in). Other commands launch external binaries: `ls`, `cat`, `cp`, `rm`, `mv`, `mkdir`. To check whether a command is built-in for a given shell, you can enter `type <command>`.[↩](#ref-f1)

<a id="f2" href="#ref-f2">2</a>: On installation, I gather that Busybox makes, for example, `/bin/ls` a symbolic link pointing to `/bin/0_shell`, allowing it act in place of a default shell. I haven't gone this far.[↩](#ref-f2)
