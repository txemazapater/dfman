# dfman

**dfman** is an experimental terminal-oriented, dual-side file operations manager.

The project explores a fast, predictable and keyboard-first way to perform common filesystem operations, taking inspiration from classic tools such as PC Tools, Norton Commander and FAR Manager while deliberately avoiding the complexity and continuous visual behaviour of modern graphical file explorers.

## Project status

**Phase 0 — architectural research and scope definition.**

No implementation language, terminal UI framework or final architecture has been selected yet.

Two reference repositories are being analysed:

- `txemazapater/FarManager` — reference for terminal interaction, keyboard-driven workflow and mature filesystem behaviour.
- `txemazapater/doublecmd` — reference for separation between file sources, operations and user interface.

These repositories are study material. dfman is not intended to be a direct fork or a reduced clone of either project.

## Initial intent

The first goal is intentionally modest: make routine operations on files and directories fast, explicit and predictable.

Candidate core operations include navigation, selection, copy, move, rename, delete and directory creation. More advanced capabilities such as search, compare, synchronization, batch rename or virtual file sources remain future possibilities rather than initial requirements.

## Architectural direction under evaluation

A promising direction identified during the initial review is to separate:

```text
Terminal UI
    |
Command / interaction layer
    |
File operation engine
    |
Filesystem / file-source abstraction
```

The terminal UI should orchestrate and present operations, not implement them.

See `docs/research/001-far-vs-doublecmd.md` for the first comparative notes.
