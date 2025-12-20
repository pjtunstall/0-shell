# Todo

- [Job control](#job-control)
  - [Names](#names)
- [General](#general)
- [Error handling](#error-handling)
- [Tests](#tests)
- [Parsing](#parsing)
- [Redirection](#redirection)
  - [Echo](#echo)
- [Further](#further)

## Job Control

- Investigate behavior of `ls -lRr / 2>1 >/dev/null  &`.
- Rename `JOB_ID` and consider name of `job.id`. Should it be `job_number` or simply `number`?
- Bash `bg` with no args stops the current job.
- Add -l and -s flags for `kill`.
- Add -n (new only) and -x (replace and execute) flags for `jobs`.
- Thoroughly check all existing and new behavior since adding elements of job-control.
  - Compare with Bash.
- Check behavior of redirection around `echo` and `cat` in conjunction with `jobs`.
- Complete the optional extra project job-control in the light of the extra requirements implied by the audit questions.
- Check formatting: sort out spacing and alignment.
- In `check_background_jobs`, check `status` for exit codes or signals (e.g., segfaults).
- Handle groups of jobs so as to allow jobs to spawn their own groups of jobs.
  - Meanwhile, ensure suitable error messages if someone tries to run a builtin fron a job.
- What should happen if `&` is the final argument for builtins?
- Write more tests for job control as I go along.
- Refactor again.
- Bring together the `has_stopped` (jobs) check into one function that can be called from `repl` and `exit`. Maybe make it a method of a `Jobs` struct.

### Naming

- Assess for consistency, especially: job, process, child.
- Root out any overly graphic terminology: e.g. favor "kill/terminate job/process" over "kill child"!

## General

- Ask for code reviews.
- Look carefully at all these refs to collections to ref types in `cat` and `ls`. Examine what they all imply and what best practice is.
- `cat`: handle mixed sequence of filenames and dashes.
- Consider structure: e.g.
  - make a `Command` struct with methods `usage`, `run`, etc. to cut down on boilerplate.
  - Make a definitive list of commands that can be shared between, e.g. `input.rs` and `man.rs` and `commands.rs` to make it harder to omit items as more are added.
  - Make a `Jobs` struct with methods for its various functionality and possibly other fields besides the the `Vec<Job>`, such as as stack to track the last two foregrounded jobs.
- Command chaining with `;`.
- Running it now on Linux, I notice that `ls` with no options formats differently to Mac. You can't please all of the people all of the time.
- Pick a consistent style of creating `String` from `&str`: either `String::from` or `to_string` or `into`.
- Consider `pipe()` and `dup2()` for pipes and redirection of file descriptors.
- Consider a crate to abstract color handling: `colored`, `owo-colors`, or `termcolor`. But, now that I'm commited to Unix-style only, maybe I've lost the main reason for choosing such a crate over plain ANSI codes.

## Error handling

- Revisit question of capitalization patterns in error messages. Internally consistent now, I hope: but what conventions to the best-known shells use?
- Feret out any remaining OS-specific error tests: e.g. ones that require a particular OS-generates error message. I think it's only custom error messages that are being compared in tests now; for system error, I think I'm just testing existence or non-existence.
- Use enums for errors so that I can test for the correct variant instead of for specific strings, thus making these tests less brittle.
- Check that error messages are consistently formatted. Maybe start to explore this when I've got tests in place to compare my commands directly against the standard shell commands. Include arguments where appropriate; see `rm`.
- Look into what happens when `ls` encounters `permission denied` errors, if that even happens.

## Tests.

- Test input functions.
- Test `ls` on Windows as is uses platform-specific code, conditional on which platform is being compiled for.
- See if there's a way to avoid some of those clones in the tests etc., e.g. `mv`. Look at whether there are places I can avoid copying, e.g. use refs instead of Strings either in the code or the tests.
- Use a loop to insert the right number of backslashes in echo special character test.
- Use this less verbose pattern in tests:

```rust
let result = cat(&input).expect("`cat` should be ok");
assert_eq!(result, "Hello, world!");`
```

- Is there any reason to prefer one above the other: creating a file then writing to it, or creating it implicitly by writing to it?
- RESEARCH: Fix test cleanup on panic. When run sequentially, the cleanup happens only in the nonpanicking thread, I think. Make a `for_test_temp_files` directory in the project root; add it to `.gitignore`. Have all test files and directories placed in there so that they can be more easily removed if cleanup fails?

## Parsing

- Handle unclosed quotes better.
- Escape special characters with backslash, especially quotes and space and backslash itself, e.g. replacing temporarily with an untypable character such as `\u{E000}`.
- Handle file and directory names that begin with a dash. (Via absolute path?) Should I escape dashes during the initial parse? See what Zsh does. How does `echo` treat dashes? A dash on its own is ignored by echo, but an initial dash followed by other characters is printed.

## Redirection

- Move redirection logic to the shell itself. Move parsing upsteam: have the shell extract redirection targets when it parses the input before passing it to the individual commands. Move the actual redirection downstream: have it write to file the ok resulting string returned by the command functions. That will means reorganizing `cat` and `ls` to handle redirection in the same way. It will need careful thought about where and when the formatting is done with `ls`.
- Make redirection logic more consistent between `cat`, `ls`, `echo`, and `jobs` so that I can call a common `redirect::redirect` function from both of them. This will mean bringing the error handling into line between these two commands: `ls` returns an ok variant that incorporates errors; `cat` returns a result that is either a dummy ok (that's not actually printed upstream) or an error containing all accumulated errors, including any redirection errors. `jobs` prints successes and failures, and repurposes the ok variant for testing. `echo` does things even more its own ad hoc way since this was the first; it will also need upgrading to handle multiple redirections.
- `exit > exit` exits Zsh and creates a file called `exit` with one blank line. My 0-shell gives an error: "Too many arguments". What's the rule? Maybe I want to do it my way.

Regarding the task of moving redirection into the shell, I get this AI advice:

```
Refactoring redirection into the shell is doable but non-trivial: today cat, ls, and echo each parse/handle redirection themselves (via redirect::separate_sources_from_targets), with slightly different behaviors. Moving it up means:

Parsing redirection tokens in the shell, stripping them from argv, opening/truncating/append files centrally, and wiring stdout/stderr accordingly before invoking the command.
Removing/redesigning per-command redirect logic (and tests) and ensuring commands return plain output strings.
Risk: moderate. Easy to break edge cases like multiple targets, append vs truncate, empty output, current “partial success with accumulated errors” semantics (cat, ls), and colored inline errors in ls. Also risk in piping/stream handling if added later.

Before refactoring, good tests to add:

- cat with multiple redirect targets (write, append) succeeds and contents match.
- ls redirect to file and append works; redirect to directory yields shell-prefixed error; non-existent paths still print inline “No such file” messages.
- echo with multiple redirect targets (write, append) mirrors behavior, preserves newline, handles quoted args.
- Mixed: command with sources + redirects, ensuring sources are untouched and redirects only use the targets.
```

### Echo

- Check exact behavior of `echo` with multiple redirect arguments: multiple spaces, etc. Write more tests.
- Check error-handling in `echo`, especially for multiple redirection targets.
- Refactor `echo`.

## Further

- Piping with `|`.
- Command chaining with `;`.
- Have a go at the other optional project, `scripting`.
- There's a hint that the project should teach the difference between Unix and Posix. Consider whether to make sure it strictly adheres to Posix. Look up Posix specifications.
- Investigate mdBook for Rust documentation.
- Consider the Nix crate for Rust abstractions over `libc`.
