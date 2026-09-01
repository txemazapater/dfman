# dfman

**dfman** is an experimental terminal-oriented, dual-side file operations manager.

The project explores a fast, predictable and keyboard-first way to perform common filesystem operations, taking inspiration from classic tools such as PC Tools, Norton Commander and FAR Manager while deliberately avoiding the complexity and continuous visual behaviour of modern graphical file explorers.

## Project status

**Phase 1 — executable Rust foundation.**

The implementation language is Rust. The repository is now a Cargo workspace with a minimal core library and CLI. The current executable only implements a deliberately small `scan` command so the toolchain, architecture and CI can be validated before native Windows filesystem optimizations or the terminal UI are introduced.

Two reference repositories remain part of the architectural research:

- `txemazapater/FarManager` — reference for terminal interaction, keyboard-driven workflow and mature Windows filesystem behaviour.
- `txemazapater/doublecmd` — reference for separation between file sources, operations and user interface.

These repositories are study material. dfman is not intended to be a direct fork or a reduced clone of either project.

## Current workspace

```text
crates/
  dfman-core/   domain and filesystem-neutral core concepts
  dfman-cli/    current command-line executable
```

Additional crates will be introduced only when the architecture requires them. Likely future boundaries include filesystem backends, operation planning/execution, journal/history, declarative DSL, natural-intent resolution and the terminal UI.

## Local setup

The repository pins Rust in `rust-toolchain.toml`. After installing Rust with `rustup` and the required Windows MSVC build tools, verify the environment with:

```text
rustc --version
cargo --version
```

Then clone or update the repository and run:

```text
cargo test
cargo run -- scan .
```

A release build can be produced with:

```text
cargo build --release
```

## First executable behaviour

The current MVP command is:

```text
dfman scan <path>
```

It builds a cheap, non-recursive `DirectorySnapshot` and reports the number of entries, files and directories. This implementation intentionally uses the Rust standard library first. A Windows-native enumerator inspired by FAR Manager will replace or complement it after the behavioural contract and benchmarks are in place.

## Architectural direction

The current conceptual flow is:

```text
Terminal UI / CLI
       |
Natural intent / declarative DSL
       |
      Basket
       |
OperationPlan
       |
 Validation
       |
 Execution
       |
  Journal / Undo
       |
Filesystem / FileSource
```

The terminal UI must orchestrate and present operations, not implement them. Natural-language interpretation, including a possible local LLM, may propose structured intent but must never access the filesystem directly.

## CI

GitHub Actions validates the workspace on Windows and Linux with:

```text
cargo fmt --check
cargo check
cargo clippy -D warnings
cargo test
cargo build --release
```

See `docs/` for the research notes, design documents and architectural decisions that define the project before implementation grows.
