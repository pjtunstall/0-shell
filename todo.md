# Todo

- [General](#general)
- [Error handling](#error-handling)
- [Tests](#tests)
- [Parsing](#parsing)
- [Redirection](#redirection)
  - [Echo](#echo)

## General

- Refactor `ls` and submodules, breaking up long functions.
- Likewise `commands.rs`.
- Ask for code reviews.
- Look carefully at all these refs to collections to ref types in `cat` and
  `ls`. Examine what they all imply and what best practice is.
- `cat`: handle mixed sequence of filenames and dashes.
- Consider structure: e.g.
  - make a `Command` struct with methods `usage`, `run`, etc. to cut down on
    boilerplate.
  - Make a definitive list of commands that can be shared between, e.g.
    `input.rs` and `man.rs` and `commands.rs` to make it harder to omit items as
    more are added.
  - Make a `Jobs` struct with methods for its various functionality and possibly
    other fields besides the the `Vec<Job>`, such as as stack to track the last
    two foregrounded jobs.
- Command chaining with `;`.
- Consider `pipe()` and `dup2()` for pipes and redirection of file descriptors.
- Consider a crate to abstract color handling: `colored`, `owo-colors`, or
  `termcolor`. But, now that I'm commited to Unix-style only, maybe I've lost
  the main reason for choosing such a crate over plain ANSI codes.
  - Piping with `|`.
- Command chaining with `;`.
- Have a go at the other optional project, `scripting`.
- There's a hint that the project should teach the difference between Unix and
  Posix. Consider whether to make sure it strictly adheres to Posix. Look up
  Posix specifications.
- Investigate mdBook for Rust documentation.
- Consider the Nix crate for Rust abstractions over `libc`.
- Change prompt to `#` for superuser.

## Error handling

- Bring the error handling into line between the various commands: `ls` returns
  an ok variant that incorporates errors; `cat` returns a result that is either
  a dummy ok (that's not actually printed upstream) or an error containing all
  accumulated errors, including any redirection errors. `jobs` prints successes
  and failures, and repurposes the ok variant for testing. `echo` does things
  even more its own ad hoc way since this was the first; it will also need
  upgrading to handle multiple redirections.
- Revisit question of capitalization patterns in error messages. Internally
  consistent now, I hope: but what conventions to the best-known shells use?
- Feret out any remaining OS-specific error tests: e.g. ones that require a
  particular OS-generates error message. I think it's only custom error messages
  that are being compared in tests now; for system error, I think I'm just
  testing existence or non-existence.
- Use enums for errors so that I can test for the correct variant instead of for
  specific strings, thus making these tests less brittle.
- Check that error messages are consistently formatted. Maybe start to explore
  this when I've got tests in place to compare my commands directly against the
  standard shell commands. Include arguments where appropriate; see `rm`.
- Look into what happens when `ls` encounters `permission denied` errors, if
  that even happens.

## Tests

- Test input functions.
- Look out for ways to test `ls`.
- See if there's a way to avoid some of those clones in the tests etc., e.g.
  `mv`. Look at whether there are places I can avoid copying, e.g. use refs
  instead of Strings either in the code or the tests.
- Use a loop to insert the right number of backslashes in echo special character
  test.
- Use this less verbose pattern in tests:

```rust
let result = cat(&input).expect("`cat` should be ok");
assert_eq!(result, "Hello, world!");`
```

- Is there any reason to prefer one above the other: creating a file then
  writing to it, or creating it implicitly by writing to it?
- RESEARCH: Fix test cleanup on panic. When run sequentially, the cleanup
  happens only in the nonpanicking thread, I think. Make a `for_test_temp_files`
  directory in the project root; add it to `.gitignore`. Have all test files and
  directories placed in there so that they can be more easily removed if cleanup
  fails?

## Parsing

- Handle unclosed quotes better.
- Escape special characters with backslash, especially quotes and space and
  backslash itself, e.g. replacing temporarily with an untypable character such
  as `\u{E000}`.
- Handle file and directory names that begin with a dash. (Via absolute path?)
  Should I escape dashes during the initial parse? See what Zsh does. How does
  `echo` treat dashes? A dash on its own is ignored by echo, but an initial dash
  followed by other characters is printed.

## Job Control

- Add -l and -s flags for `kill`.
- Add -n (new only) and -x (replace and execute) flags for `jobs`.
- Thoroughly check all existing and new behavior since adding elements of
  job-control.
- Check behavior of redirection around `echo` and `cat` in conjunction with
  `jobs`.
- Complete the optional extra project job-control in the light of the extra
  requirements implied by the audit questions.
- In `check_background_jobs`, check `status` for exit codes or signals (e.g.,
  segfaults).
- Ensure suitable error messages if someone tries to run a builtin fron a job.
- What should happen if `&` is the final argument for builtins?
- Bring together the `has_stopped` (jobs) check into one function that can be
  called from `repl` and `exit`. Maybe make it a method of a `Jobs` struct.

## Redirection

- Apply all redirections centrally in the worker, left-to-right (including
  optional leading fd and >&), remove redirect parsing from the individual
  commands (cat`, `ls`, `echo`, and `jobs`), and pass them a cleaned argv so the
  behavior matches a real shell.
- Before starting on this task, good tests to add might be:
  - cat with multiple redirect targets (write, append) succeeds and contents
    match.
  - ls redirect to file and append works; redirect to directory yields
    shell-prefixed error; non-existent paths still print inline “No such file”
    messages.
  - echo with multiple redirect targets (write, append) mirrors behavior,
    preserves newline, handles quoted args.
  - Mixed: command with sources + redirects, ensuring sources are untouched and
    redirects only use the targets.
- `exit > exit` exits Zsh and creates a file called `exit` with one blank line.
  My 0-shell gives an error: "Too many arguments". What's the rule? Maybe I want
  to do it my way.

### Echo

- Check exact behavior of `echo` with multiple redirect arguments: multiple
  spaces, etc. Write more tests.
- Check error-handling in `echo`, especially for multiple redirection targets.
- Refactor `echo`.
