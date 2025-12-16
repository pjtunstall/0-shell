# Lessons

- A pleasant workflow: write some code. Make notes. When a plan emerges, start again. Make it bit by bit.

- Write some experimental tests in parallel to the code in that first sketch, but don't aim to be comprehensive. Think how to plan it so that writing tests can precede writing code as far as possible in the main workflow. It takes discipline, but it can be satisfying when it happens. Still, it seems more natural to often write code ahead of tests in the early stages; how you choose to write that code often determines how it can be tested.

- Beware unit tests that might interfere with each other if run, as is the default, in parallel. I'm testing echo's substitution of environment variables with one test that checks the case where a variable is set and another that checks the case where it isn't. At first, I had both tests reading and writing the same variable, `$USER`. Sometimes one failed, sometimes it didn't. When the tests were run sequentially with `cargo test -- --test-threads=1` they passed reliably. After I switched to `$LANG` for one of the tests, they always passed with plain old `cargo test` too, although subsequenly, as I wrote more, other similar situations arose. I've been using `--test-threads=1` for the project since then. I did have a go at protecting shared resources--in particular the current directory--with a mutex, but there are many places where it was needed, and the problem is open-ended in the sense that anyone adding a test to any of the modules would have to remember to consider whether they needed to use the mutex.

- Don't test for potentially OS-specific error codes or messages.
  - More generally, use enums for error types and prefer to test for enum variants rather than specific strings.
