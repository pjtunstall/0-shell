# Lessons

- A pleasant workflow: write some code. Make notes. When a plan emerges, start again. Make it bit by bit.

- Write some experimental tests in parallel to the code in that first sketch, but don't aim to be comprehensive. Think how to plan it so that writing tests can precede writing code as far as possible in the main workflow.

- Beware unit tests that might interfere with each other if run, as is the default, in parallel. I'm testing echo's substitution of environment variables with one test that checks the case where a variable is set and another that checks the case where it isn't. At first, I had both tests reading and writing the same variable, `$USER`. Sometimes one failed, sometimes it didn't. When the tests were run sequentially with `cargo test -- --test-threads=1` they always passed. After I switched to `$LANG` for one of the tests, they always passed with plain old `cargo test` too.

- Don't test for potentially OS-specific error codes or messages.
