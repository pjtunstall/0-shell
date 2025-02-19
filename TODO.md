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

- Break up `ls` function, then implement for multiple dirs.
- `ls -F`.
- Handle file and directory names that begin with a dash. Should I escape dashes during the initial parse? See what Zsh does. How does `echo` treat dashes? A dash on its own is ignored by echo, but an initial dash followed by other characters is printed.
- Change `TempStore` to a Vec<String>, so that it can have any number of items, and so that they're not labeled `source` and `target`, as it's now used more generally in contexts where those names aren't relevant. Use `PathBuf` instead of `String` in `TempStore`. Tidy the tests to be consistent with this change and use `Path` more; in particular, get rid of the last remaining `{}{}{}` in `mv.rs`.
- Tests for `ls`.
- RESEARCH: Fix test cleanup on panic. When run sequentially, the cleanup happens only in the nonpanicking thread, I think.
- Check vetting of argument numbers and types in each command function. See if I can write a general function to check conditions on number of arguments (better that `check_num_args`), e.g. less than, greater than, or equal to.
- Write `USAGE` messages for all commands and look at what what triggers them; check their format is consistent.
- REFACTOR: Restructure `main` and `helpers`. Maybe add a `style` module.

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
  - Try a simple example testing against actual shell command. The ones that create or delete files don't lend themselves to that so much, but the text ones do: `cat` and `echo`, `pwd`, `ls`; and I could do one where I create a dir and `cd` into it and check `pwd` both with pure Rust and by calling the actual shell command. Would those count as integration or unit tests? Unit, I suppose.
  - Look up Posix specifications.
  - DONE: Finish all commands default option, trying to lead with tests.
  - Required option flags.
  - Test all options.
  - Unit tests for each command.
  - Integration tests.
  - See if there's a way to avoid some of those clones in the tests etc., e.g. `mv`.
  - Refactor for coinsistency of names and ways of doing things.
  - Refactor, splitting up some functions.
  - Use loop to insert the right number of backslashes in echo special character test.

## General

- Note parallels between commands (the better to structure code and tests, order creation of tests, reuse code and tests, memorize).
- Check error messages are consistently formatted, e.g. whether "os error" numerical codes are included. Maybe start to explore this when I've got tests in place to compare my commands directly against the standard shell commands. Include arguments where appropriate; see `rm`.
- Look at whether there are places I can avoid copying, e.g. use refs instead of Strings either in the code or the tests.
- Test what happens when `ls` encounters `permission denied` errors, if that even happens.

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
