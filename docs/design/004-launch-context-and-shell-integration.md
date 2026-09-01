# Design 004 — Launch context and shell integration

Status: Proposed  
Date: 2026-09-01

## Intent

dfman should be launchable not only as an interactive terminal application started from a shell, but also from external actors such as Windows Explorer, PowerShell, shortcuts, scripts, another file manager or future desktop integrations.

The executable command line therefore forms a stable boundary for injecting startup context into dfman.

The external caller does not manipulate dfman's UI directly. It describes the initial context; dfman translates that context into its own application state.

## Launch context

The startup context should be able to describe at least:

- initial current path;
- initial left panel path;
- initial right panel path;
- one or more externally selected filesystem entries;
- an optional initial dfman DSL command;
- an optional natural-language intent;
- requested startup mode, such as interactive UI, scan or benchmark.

Conceptually:

```text
External actor
    |
    v
CLI arguments
    |
    v
LaunchContext
    |
    +--> left/right panel paths
    +--> current location
    +--> initial Basket entries
    +--> optional DSL / natural intent
    |
    v
dfman application
```

## Proposed command forms

The exact grammar may evolve, but the intended interface is:

```text
dfman [path]
dfman open <path>
dfman open --left <path> --right <path>
dfman open <path> --select <entry> [--select <entry> ...]
dfman exec "<dfman DSL>"
dfman ask "<natural language intent>"
dfman scan <path>
dfman benchmark <path>
```

A bare path should eventually be equivalent to:

```text
dfman open <path>
```

This makes shell integration uncomplicated.

## Windows Explorer scenario

A first Explorer integration can invoke dfman with the folder containing the context-menu invocation:

```text
dfman.exe open "<current-folder>"
```

For example, an Explorer command conceptually equivalent to:

> Execute dfman in this location

simply passes the folder path to dfman.

A later integration may pass selected files as repeated arguments:

```text
dfman.exe open "D:\Photos" \
  --select "D:\Photos\a.jpg" \
  --select "D:\Photos\b.jpg"
```

Those selected entries should become the initial Basket rather than transient UI row selection.

This is important: Explorer supplies context; dfman retains its own Basket semantics.

## Context from the current shell

If dfman is started without an explicit path, the interactive application may use the process current working directory as its initial path:

```text
C:\Photos> dfman
```

would open dfman at `C:\Photos`.

This is different from the current diagnostic CLI, which still requires explicit subcommands while implementation is being bootstrapped.

## Two-panel startup

External callers should eventually be able to define both sides:

```text
dfman open --left "C:\Incoming" --right "D:\Archive"
```

If only one path is supplied, policy for the other panel can be configuration-driven, for example:

- same path;
- previous session path;
- user's home directory;
- configured default.

No policy is fixed by this document yet.

## Executable semantics

The command line must remain an orchestration boundary, not a second operation engine.

For example:

```text
dfman exec "MOVE TO right WHERE kind = photo"
```

must follow the same internal path as a command entered interactively:

```text
DSL
 -> Intent / AST
 -> Basket
 -> OperationPlan
 -> validation
 -> execution
 -> journal
```

Likewise:

```text
dfman ask "mueve las fotos grandes a la derecha"
```

must never give the natural-language resolver direct filesystem authority.

## Shell integration principle

> External applications provide launch context; dfman remains the sole authority that interprets that context into Basket state and operation plans.

This keeps Explorer, scripts and future integrations thin and replaceable.

## Near-term implementation

During the bootstrap phase, the CLI will remain deliberately dependency-free.

The first extension should add an `open` command that accepts a path and prints the resolved `LaunchContext`. It is a diagnostic stepping stone before the TUI exists.

Once the command surface grows enough to justify it, a dedicated argument parser such as `clap` can replace the minimal hand-written parser without changing the domain-level `LaunchContext` contract.
