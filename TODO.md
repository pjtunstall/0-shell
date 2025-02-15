# Todo

[Fix](#fix)
[General](#general)
[Strategy](#strategy)
[Command line](#command-line)
[Parsing](#parsing)
[Documentation](#documentation)
[Extras](#extras)
[Scope](#scope)

## Fix

...

## Echo

- Check error handling matches echo.
- Look up what happens if there are multiple file names.
- Debug dynamic commands https://runcloud.io/blog/echo-command-in-linux
- Write more tests, especially for redirection and special characters.

## General

- Think of the order you can create the commands in so that you can use them provisionally to test each other.
- Note parallels between commands (the better to structure code and tests, order creation of tests, reuse code and tests, memorize).
- When you have a new plan, pick a simple one to explore fully, e.g. `echo`.
- Check consistency of error messages, e.g. whether "os error" numerical codes are included.

## Strategy

- v1, using the Rust standard library.
  - DONE: Start work all in one file.
  - DONE: Prototype one or two of the simpler basic commands to see what issues will need to be solved and note ideas for how best to do the project.
  - DONE: Restructure into modules.
  - Prototype one of the simpler commands with options and variable number and type of arguments (files versus folders).
  - Look up Posix specifications.
  - Rewrite all commands with all the options I choose to implement in this first iteration.
  - Write tests for all basic commands. I could use commands to test other commands, but the usefulness of that would change if I changed the internals of how the commands work so that command A starts relying on command B, or stops doing so. There's a danger that it would give a false sense of security because of this. But maybe such dangers go with the territory of testing and are not a reason not to test.
  - Proceed one by one through the commands.

## Command line

- Ctr + C for interrupting internal processes.

## Parsing

- Think how to parse input with optional number of arguments and flags, and where items might refer to files or folders. How much can this be done in a general parsing function, and how much of it will be specific to each command?
- Replace `check_num_args` with something that deals with optional number of arguments.
- Pair single or double quotes and parse them out.
- Parse glob: `*`.

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
