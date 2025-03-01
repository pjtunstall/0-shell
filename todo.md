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

- `cat`: write more tests for redirection scenarios: different number of source and target files, mixed order, existence of target files, target files being directories, mixed existing directories and files for target files.
- Redirection for `ls`.
- `cat`: handle mixed sequence of filenames and dashes.
- Refactor `cat`: split up the pub function and flatten the nesting.
- Use this less verbose pattern in tests: `let result = cat(&input).expect("`cat` should be ok"); assert_eq!(result, "Hello, world!");`.
- Is there any reason to prefer one above the other out of creating a file then writing to it, or creating it implicitly by writing to it?
- Add a `for_test_temp_files` directory in root; add it to `.gitignore`. Have all test files and directories placed in there so that they can be more easily removed if cleanup fails.
- Add mutex to allow `cargo test`.
- Write `USAGE` messages for all commands and look at what what triggers them; check their format is consistent.
- Add `man` command.
- `echo` with multiple redirect arguments.
- Check full redirection functionality for `cat`; decide on and note any deviations from stanrad behavior, e.g. whether to allow valid operations to proceed if one source file is really a directory (why not let them go ahead to be consistent with other graceful failures, e.g. in `ls`): All arguments not preceeded by `>` or `>>` (source files) are concatenated in order into all files that are preceded by `>` or `>>` (target files). If a source file doesn't exist, it triggers a "No such file or directory" error but any other sources are concatenated; if a source is a directory, it triggers an "is a directory" error (note the inconsistent casing; no need to replicate that)--and that error stops anything from working and also prevents any "No such file or directory" errors that would have occurred. If a target file doesn't exist, it's created. If there are no target files, the output is sent to stdout. `cat` with no source arguments waits for input from stdin; if there are target arguments, it creates them immediately if they don't exists, but only writes to them when it encounters EOF. It exits with Ctr + D or Ctrl + C.
- Switch `echo` redirection tests to use `TempStore`.
- Change the `get_input` input function in `cat` to use termion for greater control, of keyboard shortcuts and interrupts, especially Ctr + C.

## Fix?

- `exit > exit` exits Zsh and creates a file called `exit` with one blank line. My 0-shell gives an error: Too many arguments. What's the rule? Maybe I want to do it my way.

## General

- Test input functions.
- Test `ls` on Windows as is uses platform-specific code, conditional on which platform is being compiled for.
- See if there's a way to avoid some of those clones in the tests etc., e.g. `mv`. Look at whether there are places I can avoid copying, e.g. use refs instead of Strings either in the code or the tests.
- Use a loop to insert the right number of backslashes in echo special character test.
- Check error messages are consistently formatted. Maybe start to explore this when I've got tests in place to compare my commands directly against the standard shell commands. Include arguments where appropriate; see `rm`.
- Feret out any remaining OS-specific error tests: e.g. ones that require a particular OS-generates error message. I think it's only custom error messages that are being compared in tests now; for system error, I think I'm just testing existence or non-existence.
- Test what happens when `ls` encounters `permission denied` errors, if that even happens.
- Add redirection for `ls`.
- Scripting.
- RESEARCH: Fix test cleanup on panic. When run sequentially, the cleanup happens only in the nonpanicking thread, I think.

## Command line

- Ctr + C for interrupting internal processes. And implement some internal process that might take significant time so that Ctr + C can be seen in action.

## Parsing

- Handle unclosed quotes better.
- Escape special characters with backslash, especially quotes and space and backslash itself, e.g. replacing temporarily with an untypable character such as `\u{E000}`.
- Handle file and directory names that begin with a dash. (Via absolute path?) Should I escape dashes during the initial parse? See what Zsh does. How does `echo` treat dashes? A dash on its own is ignored by echo, but an initial dash followed by other characters is printed.

## Documentation

- Investigate mdBook for Rust documentation.
- Write a manual of usage in a standardized form, accessible from the command line.

## Scope

- Consider the two optional extra projects, `0-shell-job-control` and `0-shell-scripting`. (For what it's worth, 4 weeks are assigned to the main project and 2 weeks for each of the extras. All three projects are for 4 students.)
- There is a hint that the project should teach the difference between Unix and Posix. Consider whether it should adhere to Posix. Look up Posix specifications.
