# Todo

- Think of the order you can create the commands in so that you can use them provisionally to test each other.
- Note parallels between commands.
- When you have a new plan, pick a simple one to explore fully, e.g. `echo`.

## Command line

- Implement up and down arrows to cycle through history, perhaps using DeqVec. Will I need an async runtime to capture inpute while waiting for stdin?
- Implement variables.

## Parsing

- Think how to parse input with optional number of arguments and flags, and where items might refer to files or folders. How much can this be done in a general parsing function, and how much of it will be specific to each command?
- Replace `check_num_args` with something that deals with optional number of arguments.
- Pair single or double quotes and parse them out.
- Parse glob: `*`.

## Documentation

- Investigate mdBook for Rust documentation.
- Write a manual of usage in a standardized form, accessible from the command line.
- Mini usage messages for each command, for when its arguments can't be parsed.

## Extras

- `touch`.

## Scope

- Define the scope of the project. There are two main options: syscalls and basic. Do the basic one first: as practice at Rust + structuring + planning, and to get the course credits. That will define the interface and input handling, which would be necessary for the syscall version too. The syscall option can be added later or not at all. The syscall option would be more interesting and educational and impressive, but the end result in either case would be simply to recreate some of the functionality of already existing, reliable programs; they'd both just be learning exercises, with nothing unique to show for it at the end, unless you can think of a gimmick?
