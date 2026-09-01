# dfman

**dfman** is an experimental terminal-oriented, dual-side file operations manager.

The project explores a fast, predictable and keyboard-first way to perform common filesystem operations, taking inspiration from classic tools such as PC Tools, Norton Commander and FAR Manager while deliberately avoiding the complexity and continuous visual behaviour of modern graphical file explorers.

## Project status

**Phase 1 — executable Rust foundation.**

The implementation language is Rust. The repository is a Cargo workspace with a minimal core library and CLI. The current executable deliberately exposes only a small surface (`scan` and `open`) so the toolchain, launch context, architecture and CI can be validated before native Windows filesystem optimizations or the terminal UI are introduced.

Two reference repositories remain part of the architectural research:

- `txemazapater/FarManager` — reference for terminal interaction, keyboard-driven workflow and mature Windows filesystem behaviour.
- `txemazapater/doublecmd` — reference for separation between file sources, operations and user interface.

These repositories are study material. dfman is not intended to be a direct fork or a reduced clone of either project.

## Current workspace

```text
crates/
  dfman-core/   domain and filesystem-neutral core concepts
  dfman-cli/    current command-line executable

scripts/windows/
  bootstrap-local.ps1     initialize a freshly cloned local development copy
  update-local.ps1        git pull, test and reinstall dfman locally
  install-explorer.ps1    register Explorer context-menu integration
  uninstall-explorer.ps1  remove Explorer context-menu integration
```

Additional crates will be introduced only when the architecture requires them. Likely future boundaries include filesystem backends, operation planning/execution, journal/history, declarative DSL, natural-intent resolution and the terminal UI.

## Local setup

The repository pins Rust in `rust-toolchain.toml`. After installing Rust with `rustup` and the required Windows MSVC build tools, verify the environment with:

```text
rustc --version
cargo --version
```

For a freshly cloned repository on Windows:

```powershell
.\scripts\windows\bootstrap-local.ps1
```

This checks the required tools, formats and tests the workspace, and installs `dfman.exe` through Cargo. With the normal rustup setup the executable is placed in `%USERPROFILE%\.cargo\bin`, which is normally already in `PATH`.

Verify with:

```powershell
where.exe dfman
dfman scan .
dfman open .
```

## Updating the local executable

Once the repository is already cloned, the normal local update cycle is:

```powershell
.\scripts\windows\update-local.ps1
```

The script performs:

```text
git pull --ff-only
cargo fmt --all
cargo test --workspace
cargo install --path crates/dfman-cli --force
```

After it completes, the globally available `dfman` command points to the newly built local version.

## Windows Explorer integration

After `dfman` has been installed into Cargo's bin directory, register the experimental Explorer integration with:

```powershell
.\scripts\windows\install-explorer.ps1
```

The registration is made only for the current user under `HKCU\Software\Classes`; administrator privileges are not required.

It currently adds context-menu actions for:

```text
Directory               -> Open in dfman
Directory background    -> Open dfman here
```

The actions launch the same executable and pass the selected/current directory as launch context:

```text
dfman open <path>
```

On Windows 11 these classic shell entries may initially appear under **Show more options**.

To remove the integration completely:

```powershell
.\scripts\windows\uninstall-explorer.ps1
```

## Current executable behaviour

The current commands are:

```text
dfman scan <path>
dfman open <path> [--left <path>] [--right <path>] [--select <path>]...
```

`scan` builds a cheap, non-recursive `DirectorySnapshot` and reports the number of entries, files and directories. This implementation intentionally uses the Rust standard library first. A Windows-native enumerator inspired by FAR Manager will replace or complement it after the behavioural contract and benchmarks are in place.

`open` builds and displays the initial `LaunchContext`. It is the contract that will later initialize the TUI from Explorer, a shell, another application or automation.

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
