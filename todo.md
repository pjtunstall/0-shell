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

There's a lot to read here, but to pass the audit for job-control, it needs to launch arbitrary external binaries (apart from those we had to re-implement). I also need to handle + and - properly; the current program takes a short cut and just defines them as the last two items entered into the jobs list.

- History stack to track 1st (+) and 2nd (-) most recently active jobs.
- Rename `JOB_ID` and consider name of `job.id`. Should it be `job_number` or simply `number`?
  - Allow other sorts of `jobspec`: %%, %-, +/i for `bg` and `fg`.
    - When those are implemented, change usage messages to have the more general `jobspec` in place of `JOB_ID`.
- Bash `bg` with no args stops the current job.
- Add -l and -s flags for `kill`.
- Add -n flag for `jobs`.
- Re. `fg`: "Currently no terminal control (tcsetpgrp), so it doesn’t truly foreground in the Bash sense (though you forward signals via CURRENT_CHILD_PID)". Is this relevant?
- Thoroughly check all existing and new behavior since adding elements of job-control.
  - Compare with Bash.
- Check behavior of redirection around `echo` and `cat` in conjunction with `jobs`.
- Check on different platforms; for now, only tried on Linux.
- Complete the optional extra project job-control in the light of the extra requirements implied by the audit questions.
- Check formatting: sort out spacing and alignment.
- In `check_background_jobs`, check `status` for exit codes or signals (e.g., segfaults).
- Handle groups of jobs so as to allow jobs to spawn their own groups of jobs.
  - Meanwhile, ensure suitable error messages if someone tries to run a builtin fron a job.
- What should happen if `&` is the final argument for builtins?
- Remove `Terminated` state from `jobs::State` enum?
- Write more tests for job control as I go along.

A more detailed review of the job-control commands follows:

### Kill

_kill deviates from Bash (src/commands/kill.rs): only supports SIGTERM; no -SIGNAL/-l/-s flags; rejects PIDs not tracked as jobs (Bash happily kills arbitrary PIDs); forbids PID 0/negative (Bash allows -n for process groups). If you plan to mimic Bash, broaden to arbitrary PIDs and signals, and don’t gate on the jobs table._

Preventing the killing of processes not created from within 0-shell was an intentional safety measure during development; now removed.

### bg

_bg default-job behavior missing (src/commands/bg.rs): Bash bg with no args resumes the “current” stopped job; here it errors for missing args. Also doesn’t recognize job specs like %%, %-, +/-, or command-prefix matches_

### Jobs

_jobs output/signs differ (src/commands/jobs.rs): plus/minus markers are assigned by position in the vector, not recency; filtering (jobs 2) still uses the original index, so signs can be wrong vs Bash’s %+/%-. Job specs are limited to numeric (no %%, %-, +/-, or name matching). Options partially match: -p is pid-only (ok), -l adds PID (ok), but combinations are Bash-like only by accident and there’s no support for -n (new only) or -x (replace and execute)._

### Job table updates

_Job table updates:_ **check_background_jobs** _runs on entry to each command and cleans up with waitpid(WNOHANG|WUNTRACED), which is fine for the current design. It prints status lines directly; Bash writes to the terminal too, so that’s acceptable._

### fg

_fg job-spec coverage is partial (src/commands/fg.rs): falls back to last job if none given (good), but only accepts numeric IDs (with optional %). Bash accepts %%, %-, +/-, and job-name prefixes._ _No terminal control (tcsetpgrp), so it doesn’t truly foreground in the Bash sense (though you forward signals via_ **CURRENT_CHILD_PID**).

### Groups

_Process groups/terminal control: Without process groups you can’t fully mimic Bash job control (no proper foreground ownership, Ctrl+C handling, or kill to a job’s PGID). If “mimic Bash” is the goal, adding process groups and tcsetpgrp would be necessary later._

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
