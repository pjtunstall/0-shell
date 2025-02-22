# Todo

- [Next](#next)
- [Fix](#fix)
- [General](#general)
- [Command line](#command-line)
- [Parsing](#parsing)
- [Documentation](#documentation)
- [Extras](#extras)
- [Scope](#scope)

## Next

- Make `ls` fully cross-platform.
- Handle file and directory names that begin with a dash. (Via absolute path?) Should I escape dashes during the initial parse? See what Zsh does. How does `echo` treat dashes? A dash on its own is ignored by echo, but an initial dash followed by other characters is printed.
- Write `USAGE` messages for all commands and look at what what triggers them; check their format is consistent.
- RESEARCH: Fix test cleanup on panic. When run sequentially, the cleanup happens only in the nonpanicking thread, I think.
- REFACTOR: Restructure `main` and `helpers`.

## Fix?

- `exit > exit` exits Zsh and creates a file called `exit` with one blank line. My 0-shell gives an error: too many arguments. What's the rule? Maybe I want to do it my way.

## General

- Look up Posix specifications.
- See if there's a way to avoid some of those clones in the tests etc., e.g. `mv`. Look at whether there are places I can avoid copying, e.g. use refs instead of Strings either in the code or the tests.
- Refactor for coinsistency of names and ways of doing things.
- Refactor, splitting up some functions.
- Use a loop to insert the right number of backslashes in echo special character test.
- Check error messages are consistently formatted. Maybe start to explore this when I've got tests in place to compare my commands directly against the standard shell commands. Include arguments where appropriate; see `rm`.
- Remove any remaining OS-specific error tests: e.g. ones that require a particular OS-generates error message.
- Test what happens when `ls` encounters `permission denied` errors, if that even happens.
- `ls -l`: look carefully at all that formatting and refactor if some is superfluous.
- Note parallels between commands (the better to structure code and tests, order creation of tests, reuse code and tests, memorize).
- Check vetting of argument numbers and types in each command function. See if I can write a general function to check conditions on number of arguments (better that `check_num_args`), e.g. less than, greater than, or equal to.

## Command line

- Ctr + C for interrupting internal processes.

## Parsing

- Replace `check_num_args` with something that deals with optional number of arguments.
- Handle unclosed quotes better.
- Escape special characters with backslash, especially quotes and spaces, e.g. replacing temporarily with an untypable character such as `\u{E000}`.

## Documentation

- Investigate mdBook for Rust documentation.
- Write a manual of usage in a standardized form, accessible from the command line.
- Mini usage messages for each command, for when its arguments can't be parsed.

## Extras

- Add redirection for `ls`.

## Scope

- Consider the two optional extra projects, `0-shell-job-control` and `0-shell-scripting`. (For what it's worth, 4 weeks are assigned to the main project and 2 weeks for each of the extras. All three projects are for 4 students.)
- There is a hint that the project should teach the difference between Unix and Posix. Consider whether it should adhere to Posix.
- "You must program a mini Unix shell, try to focus on something simple like BusyBox." What, if anything, is special about BusyBox that should warrent this mention?
