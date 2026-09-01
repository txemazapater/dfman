# Research 001 — FAR Manager vs Double Commander

Status: initial investigation  
Date: 2026-09-01

## Purpose

Identify architectural ideas worth carrying into dfman before choosing implementation technology or writing application code.

The objective is not to decide which existing project is "better". FAR Manager and Double Commander solve related problems with substantially different architectures and UI models. We want to identify useful boundaries and avoid inheriting unnecessary complexity.

## FAR Manager — initial observations

FAR is the stronger behavioural and interaction reference for dfman:

- terminal-oriented interaction;
- keyboard-first operation;
- dual-panel workflow;
- mature handling of real filesystem edge cases;
- compact, information-dense presentation.

However, the initial source review shows that some core file operations are relatively aware of the surrounding application. For example, FAR's copy implementation (`far/copy.cpp`, `ShellCopy`) directly references panels, dialogs, filters, progress presentation and other application services in addition to filesystem primitives.

This is not a criticism of FAR: it is a mature integrated application. It does mean that directly extracting its operation engine is unlikely to give dfman the clean separation we want.

### What to learn from FAR

- interaction model;
- keyboard conventions;
- terminal behaviour;
- filesystem edge cases;
- copy/move conflict policies;
- efficient presentation of large directory listings.

### What not to assume

- that FAR's internal application structure should become dfman's architecture;
- that its complete feature set belongs in dfman;
- that terminal UI concerns should leak into filesystem operations.

## Double Commander — initial observations

Double Commander is particularly interesting architecturally.

Its `IFileSource` / `TFileSource` abstraction separates the concept of a source of files from the concrete local filesystem. File sources expose capabilities and create operation objects for listing, copying, moving, deleting, creating directories, checksums, statistics and other actions.

The operation model (`TFileSourceOperation`) also has an explicit lifecycle with states such as not started, running, paused, waiting for feedback, waiting for a connection and stopped. Operations expose progress and communicate with assigned user interfaces rather than being identical to a particular dialog or panel.

This is much closer to the kind of boundary dfman should investigate.

### Particularly interesting concepts

```text
FileSource
  -> capabilities
  -> enumerate / retrieve properties
  -> create operation

FileOperation
  -> lifecycle
  -> progress
  -> pause / stop
  -> request feedback
  -> result

UI
  -> invokes operations
  -> displays state
  -> answers operation questions
```

Double Commander also demonstrates why a file-source abstraction can be valuable later: local filesystem, archives, search results, recycle bin and other sources can participate in a broadly common model.

### Caveat

Double Commander is a large GUI application and its abstractions include GUI/LCL concerns that dfman does not necessarily need. We should extract the architectural idea, not reproduce the class hierarchy.

## Preliminary conclusion

The two references appear complementary rather than competing:

```text
FAR Manager
    -> interaction and behaviour reference

Double Commander
    -> architectural separation reference

                    dfman
                      |
          +-----------+-----------+
          |                       |
    terminal UX              clean operation core
```

The strongest initial design principle is therefore:

> A file operation must not depend on a panel, terminal widget or dialog.

A second useful principle is:

> The UI observes and controls operations; it does not own their implementation.

## Candidate minimal boundary

Without committing to classes, language or framework, the following conceptual boundary is worth prototyping:

```text
FileSource
  list(path)
  stat(path)
  capabilities()

Operation
  start()
  cancel()
  state
  progress
  feedback request
  result

Operations
  copy(source-set, destination)
  move(source-set, destination)
  delete(source-set)
  mkdir(path)
  rename(source, destination)
```

The first implementation should probably support only the local filesystem. The abstraction should make additional sources possible without requiring them in the MVP.

## Questions for the next research pass

1. How does Double Commander implement the concrete local filesystem source and its copy/move/delete operations?
2. How much of its operation queue/threading model is useful for dfman, and how much is accidental complexity?
3. How does FAR enumerate and refresh directory panels, especially with very large directories?
4. Which Windows filesystem APIs are used by FAR and Double Commander for copy/move/delete and metadata retrieval?
5. Can dfman's panel model be snapshot-based by default rather than continuously watching directories?
6. What should happen when the filesystem changes externally while a snapshot is displayed?
7. Which conflict/error policies are essential in the first version?
8. Only after answering these questions: which implementation language and terminal UI technology best fit the resulting architecture?
