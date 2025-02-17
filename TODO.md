# Todo

[Next](#next)
[Fix](#fix)
[General](#general)
[Strategy](#strategy)
[Command line](#command-line)
[Parsing](#parsing)
[Documentation](#documentation)
[Extras](#extras)
[Scope](#scope)

## Next

- Complete the unit tests for `cp`.
- Add unhappy tests for `mv`.
- Should Rust expected messages have quote code in matching backticks or single quotes or an opening backtick and a closing single quote?
- `rm` basic.
- Options.

## Fix

- `exit > exit` exits Zsh and creates a file called `exit` with one blank line. My 0-shell gives an error: too many arguments. What's the rule?
- Handle cases like the following. Zsh:

```zsh
% cd file copy
cd: string not in pwd: file
```

Or this one.

```zsh
(base) petertunstall@Peters-Air-2 0-shell % cd destination blah
cd: string not in pwd: destination
(base) petertunstall@Peters-Air-2 0-shell % cd destination
(base) petertunstall@Peters-Air-2 destination %
```

Or this.

```zsh
(base) petertunstall@Peters-Air-2 0-shell % cd file filius destination
cd: too many arguments
```

- And another misleading one with `mkdir`:

```zsh
(base) petertunstall@Peters-Air-2 0-shell % mkdir dest
mkdir: dest: File exists
```

We can do better. And look at the inconsistent punctuation:

```zsh
(base) petertunstall@Peters-Air-2 0-shell % cp dest source
cp: dest is a directory (not copied).
```

## Strategy

- v1, using the Rust standard library.
  - DONE: Start work all in one file.
  - DONE: Prototype one or two of the simpler basic commands to see what issues will need to be solved and note ideas for how best to do the project.
  - DONE: Restructure into modules.
  - DONE: Prototype one of the simpler commands with options and variable number and type of arguments (files versus folders). (echo)
  - Try a simple example of option handling.
  - Try a simple example testing against actual shell command. The ones that create or delete files don't lend themselves to that so much, but the text ones do: `cat` and `echo`, `pwd`, `ls`; and I could do one where I create a dir and `cd` into it and check `pwd` both with pure Rust and by calling the actual shell command.
  - Look up Posix specifications.
  - Finish all commands default option, trying to lead with tests.
  - Required option flags.
  - Test all options.
  - Unit tests for each command.
  - Integration tests.

## General

- Note parallels between commands (the better to structure code and tests, order creation of tests, reuse code and tests, memorize).
- Check consistency of error messages, e.g. whether "os error" numerical codes are included. Maybe start to explore this when I've got tests in place to compare my commands directly against the standard shell commands.

## Tests

- Test each command module independently, prioritizing the public functions.
- I was forgetting the obvious: besides the pure Rust tests, try testing against actual shell commands using `std::process::Command;`. Try this on the next command I work on.
- Cross test commands in integration tests.
- Think of the order you can create the commands in so that you can use them provisionally to test each other. That goes in integration tests. Not such a priority now that I have a clearer unit-testing strategy.

## Command line

- Ctr + C for interrupting internal processes.

## Parsing

- Think how to parse input with optional number of arguments and flags, and where items might refer to files or folders. How much can this be done in a general parsing function, and how much of it will be specific to each command?
- Replace `check_num_args` with something that deals with optional number of arguments.
- Pair single or double quotes and parse them out.
- Consider how to handle flags that can appear in different positions or combinations. Check the rules.

## Documentation

- Investigate mdBook for Rust documentation.
- Write a manual of usage in a standardized form, accessible from the command line.
- Mini usage messages for each command, for when its arguments can't be parsed.

## Extras

- `touch`.

## Scope

- Consider the two optional extra projects, `0-shell-job-control` and `0-shell-scripting`.
- 4 weeks are assigned to the main project and 2 weeks for each of the extras. All three projects are for 4 students.
- There is a hint that the project should teach the difference between Unix and Posix. Consider whether it should adhere to Posix.
- "You must program a mini Unix shell, try to focus on something simple like BusyBox." What, if anything, is special about BusyBox that should warrent this mention?

## Explore

- Look into peek and peekable.
