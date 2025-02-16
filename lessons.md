# Lessons

## Sketch

Write some code. Make notes. When a plan emerges, start again. Make it bit by bit.

Write tests in parallel to the code in that first sketch. Think how to plan it so that writing tests can precede writing code as far as possible in the main workflow.

- It was necessary to set tests to run serially to avoid confusing those that changed the same environment variable. This is what the `.cargo.config.toml` is for.
