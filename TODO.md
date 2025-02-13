# Todo

[General](#general)
[Strategy](#strategy)
[Command line](#command-line)
[Parsing](#parsing)
[Documentation](#documentation)
[Extras](#extras)
[Scope](#scope)

## General

- Think of the order you can create the commands in so that you can use them provisionally to test each other.
- Note parallels between commands (the better to structure code and tests, order creation of tests, reuse code and tests, memorize).
- When you have a new plan, pick a simple one to explore fully, e.g. `echo`.

## Strategy

- v1, using the Rust standard library.
  - DONE: Start work all in one file.
  - DONE: Prototype one or two of the simpler basic commands to see what issues will need to be solved and note ideas for how best to do the project.
  - DONE: Restructure into modules.
  - Prototype one of the simpler commands with options and variable number and type of arguments (files versus folders).
  - Decide on scope of v1 (non-syscall). Look up Posix specifications.
  - Rewrite all commands with all the options I choose to implement in this first iteration.
  - Write tests for all basic commands. I could use commands to test other commands, but the usefulness of that would change if I changed the internals of how the commands work so that command A starts relying on command B, or stops doing so. There's a danger that it would give a false sense of security because of this. But maybe such dangers go with the territory of testing and are not a reason not to test.
- v2, using syscalls.
  - Write trivial tests that use each command to test itself; this will be become useful if I switch to using syscalls. One could argue that before writing syscall-only versions of the functions, most tests would be trivially testing `std` functions anyway. The redundant tests proposed initially would at least have the virtue of clarity when it comes to comparing them with the syscall versions of functions.
  - Try implementing one of the simpler commands using syscalls. Test on all the main OSs: old Mac, new Mac, Linux, and Windows.
  - Proceed one by one through the commands.

## Command line

- Implement up and down arrows to cycle through history, perhaps using DeqVec. Will I need an async runtime to capture inpute while waiting for stdin?
- Implement variables.

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

- Define the scope of the project. There are two main options: syscalls and basic. Do the basic one first: as practice at Rust + structuring + planning, and to get the course credits. That will define the interface and input handling, which would be necessary for the syscall version too. The syscall option can be added later. The syscall option would be more interesting and educational and impressive, although the end result in either case would be simply to recreate some of the functionality of already existing, reliable programs. They'd both just be learning exercises, with nothing unique to show for it at the end, unless you can think of a gimmick?
- Consider the two optional extra projects, `0-shell-job-control` and `0-shell-scripting`.
- 4 weeks are assigned to the main project and 2 weeks for each of the extras. All three projects are for 4 students.
- There is a hint that the project should teach the difference between Unix and Posix. Consider whether it should adhere to Posix.
- "You must program a mini Unix shell, try to focus on something simple like BusyBox." What, if anything, is special about BusyBox that should warrent this mention?
