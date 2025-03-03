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

- Write a test for redirect.
- Switch `echo` redirection tests to use `TempStore`.
- Remove `echo` tests that expect there to not be multiple redirection targets!
- Implement `echo` with multiple redirect arguments.
- Remove superfluous inclusion of command in test function names.
- Refactor `ls`.
- Refactor `cat`: split up the pub function and flatten the nesting.
- Look carefully at all these refs to collections to ref types in `cat` and `ls`. Examine what they all imply and what best practice is.
- Replace unwraps with graceful error ballet in `cat` and `ls` redirect.
- Make redirection logic more consistent between `cat` and `ls` so that I can call a common `redirect::redirect` function from both of them. This will mean bringing the error handling into line between these two commands: `ls` returns an ok result that incorporates errors; `cat` returns a result that is either a dummy ok (that's not actually printed by `main`) or an error containing all accumulated errors, including any redirection errors. `echo` does things even more its own ad hoc way since this was the first; it will also need upgrading to handle multiple redirections.
- `cat`: write more tests for redirection scenarios: different number of source and target files, mixed order, existence of target files, target files being directories, mixed existing directories and files for target files.
- `cat`: handle mixed sequence of filenames and dashes.
- Use this less verbose pattern in tests: `let result = cat(&input).expect("`cat` should be ok"); assert_eq!(result, "Hello, world!");`.
- Is there any reason to prefer one above the other: creating a file then writing to it, or creating it implicitly by writing to it?
- Write `USAGE` messages for all commands and look at what what triggers them; check their format is consistent.
- Add `man` command.
- Change the `get_input` input function in `cat` to use termion for greater control, of keyboard shortcuts and interrupts, especially Ctr + C.
- Eventually move redirection logic to the shell itself. Move parsing upsteam: have the shell extract redirection targets when it parses the input before passing it to the individual commands. Move the actual redirection downstream: have it write to file the ok resulting string returned by the command functions. That will means reorganizing `cat` and `ls` to handle redirection in the same way. It will need careful thought about where and when the formatting is done with `ls`.

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
- DONE: Consider how to handle redirection if `ls` when there's an error. Gemini: "The ls command, when it encounters errors, sometimes elects to suppress or not produce standard output. This is a design choice within the ls command itself. This behavior is not universally true for all commands, and it can vary between different versions of ls." At the moment, I'm still redirecting it. I don't know if that's right; I thought redirection was the responsibility of the shell. For now I'm letting failures not abort, thus more like how `cat` does it. I think I'm happy with that.
- Scripting.
- RESEARCH: Fix test cleanup on panic. When run sequentially, the cleanup happens only in the nonpanicking thread, I think. Make a `for_test_temp_files` directory in the project root; add it to `.gitignore`. Have all test files and directories placed in there so that they can be more easily removed if cleanup fails?

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
