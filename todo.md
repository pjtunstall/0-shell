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

- Write `USAGE` messages for all commands and look at what what triggers them; check their format is consistent.

## Fix?

- `exit > exit` exits Zsh and creates a file called `exit` with one blank line. My 0-shell gives an error: too many arguments. What's the rule? Maybe I want to do it my way.

## General

- Test input functions.
- Test `ls` on Windows as is uses platform-specific code, conditional on which platform is being compiled for.
- See if there's a way to avoid some of those clones in the tests etc., e.g. `mv`. Look at whether there are places I can avoid copying, e.g. use refs instead of Strings either in the code or the tests.
- Use a loop to insert the right number of backslashes in echo special character test.
- Check error messages are consistently formatted. Maybe start to explore this when I've got tests in place to compare my commands directly against the standard shell commands. Include arguments where appropriate; see `rm`.
- Remove any remaining OS-specific error tests: e.g. ones that require a particular OS-generates error message.
- Test what happens when `ls` encounters `permission denied` errors, if that even happens.
- Check vetting of argument numbers and types in each command function. See if I can write a general function to check conditions on number of arguments (better than `check_num_args`), e.g. less than, greater than, or equal to.
- Add redirection for `ls`.
- Scripting.
- RESEARCH: Fix test cleanup on panic. When run sequentially, the cleanup happens only in the nonpanicking thread, I think.

## Command line

- Ctr + C for interrupting internal processes. And implement some internal process that might take significant time so that Ctr + C can be seen in action.

## Parsing

- Replace `check_num_args` with something that deals with optional number of arguments.
- Handle unclosed quotes better.
- Escape special characters with backslash, especially quotes and space and backslash itself, e.g. replacing temporarily with an untypable character such as `\u{E000}`.
- Handle file and directory names that begin with a dash. (Via absolute path?) Should I escape dashes during the initial parse? See what Zsh does. How does `echo` treat dashes? A dash on its own is ignored by echo, but an initial dash followed by other characters is printed.

## Documentation

- Investigate mdBook for Rust documentation.
- Write a manual of usage in a standardized form, accessible from the command line.

## Scope

- Consider the two optional extra projects, `0-shell-job-control` and `0-shell-scripting`. (For what it's worth, 4 weeks are assigned to the main project and 2 weeks for each of the extras. All three projects are for 4 students.)
- There is a hint that the project should teach the difference between Unix and Posix. Consider whether it should adhere to Posix. Look up Posix specifications.
