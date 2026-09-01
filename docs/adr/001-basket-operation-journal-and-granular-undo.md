# ADR-001 — Basket, operation journal and granular undo

Status: Accepted  
Date: 2026-09-01

## Context

dfman is intended to operate primarily on subsets of filesystem entries rather than always on complete directory contents.

A central concept called the **Basket** represents the materialized set of entries that the user intends to manipulate. Selection is therefore treated as an operation that modifies the Basket rather than as transient UI state attached to panel rows.

Operations such as move, rename, delete, copy and similar actions act on a Basket and are first materialized as an `OperationPlan` before execution.

A simple last-operation undo would underuse the information already available in the Basket and OperationPlan. If execution is journaled at item level, the history can support substantially more precise recovery.

## Decision

### 1. Basket is an explicit domain object

A Basket contains the concrete filesystem entries that form the scope of a future operation.

It is independent from:

- cursor position;
- current sort order;
- visual filters;
- panel repainting;
- directory refresh;
- the current visible subset of the snapshot.

Conceptually:

```text
Snapshot
   |
selection expression
   |
   v
Basket
```

### 2. Operations are planned before execution

An operation over a Basket is materialized as an `OperationPlan`.

The plan identifies at least:

- operation type;
- participating Basket entries;
- source identities and paths;
- intended destination or transformation;
- conflict policy;
- preconditions required for safe execution.

The plan may be validated and previewed before execution.

### 3. Execution produces an item-level journal

The history is not a textual activity log. It is an operational journal.

Each operation receives a stable operation identifier and stores enough information to describe the outcome of each participating entry independently.

Example:

```text
operation: OP-1042
type: MOVE

items:
  001  C:\Photos\a.jpg -> D:\Archive\a.jpg  completed
  002  C:\Photos\b.jpg -> D:\Archive\b.jpg  completed
  003  C:\Photos\c.jpg -> D:\Archive\c.jpg  failed
```

Useful item states include, at minimum:

```text
planned
started
completed
failed
rolled_back
undo_failed
```

The exact state machine will be defined separately.

### 4. Undo is generated from journaled facts

Undo is not implemented as a collection of ad-hoc UI actions.

When an operation is reversible, dfman generates an inverse `OperationPlan` from the journal and validates that inverse plan against the current filesystem state before executing it.

Examples:

```text
MOVE A -> B       => MOVE B -> A
RENAME A -> B     => RENAME B -> A
COPY A -> B       => DELETE B, if B still matches the produced copy
MKDIR A           => RMDIR A, if the directory remains safely removable
DELETE A          => RESTORE A, when deletion used dfman-managed reversible storage
```

Operations may be classified as:

```text
reversible
conditionally reversible
irreversible
```

### 5. History supports granular undo

Because the journal stores item-level results, undo does not have to be limited to an all-or-nothing LIFO stack.

dfman should be able to address:

- the complete last operation;
- a specific historical operation;
- only the successful part of a partially completed operation;
- an explicitly selected subset of entries from one historical operation.

Conceptually:

```text
history

OP-1042  MOVE    347 items
OP-1041  RENAME   42 items
OP-1040  COPY     18 items
```

Possible semantics may later include forms such as:

```text
undo
undo OP-1041
undo OP-1042 failed
undo OP-1042 item 12..20
```

The final command grammar is intentionally not fixed by this ADR.

### 6. Granular undo is dependency-aware

Historical operations are not assumed to be independent.

Example:

```text
OP-100  MOVE   A -> B
OP-101  RENAME B -> C
```

Undoing `OP-100` directly cannot blindly execute `B -> A`, because `B` no longer exists and `C` may represent the same object after a later operation.

Therefore a requested historical undo must be validated against subsequent journal entries and the current filesystem state.

The engine may classify an undo request as:

```text
safe
safe with adjusted inverse plan
conflicting
impossible
```

This is essential for non-LIFO and granular undo.

### 7. Undo itself is journaled

An undo is an ordinary operation produced by an inverse plan and therefore generates its own journal entry.

This avoids special hidden state and makes redo a consequence of the same model rather than a separate mechanism.

Conceptually:

```text
OperationPlan
    |
 execute
    v
Journal entry
    |
 inverse plan
    v
Undo operation
    |
 execute
    v
Journal entry
```

### 8. Reversible delete should be controlled by dfman

Where practical, the normal `delete` operation should use dfman-managed reversible storage or an equivalent mechanism that retains reliable original-location metadata.

A distinct explicitly irreversible operation, tentatively `purge`, may permanently destroy entries.

The exact storage strategy, retention policy and cross-volume behaviour will be defined separately.

## Consequences

### Positive

- The Basket becomes the common unit for selection, execution, retry and recovery.
- Partial failures can be understood and recovered without reconstructing history heuristically.
- Undo can operate at operation level or item level.
- Redo naturally follows from journaling inverse operations.
- The user can inspect what dfman actually changed rather than relying on opaque filesystem side effects.
- Batch operations become safer because their exact scope and outcomes remain addressable.

### Costs

- Persistent operation history becomes part of the core architecture rather than an optional UI feature.
- Entry identity must be stronger than path alone where the platform allows it.
- Non-LIFO undo requires dependency and conflict analysis.
- Journal consistency and crash recovery become important design concerns.
- Reversible delete requires managed storage and retention rules.

## Architectural principle

> The Basket defines the scope of an operation; the journal records the facts of its execution; undo is a validated inverse operation over those facts.

A related principle follows:

> History in dfman is executable history, not merely a log.

## Follow-up decisions

Separate design work is required for:

1. filesystem entry identity and validation;
2. Basket lifecycle and persistence;
3. operation and item state machines;
4. journal persistence and crash consistency;
5. dependency analysis between historical operations;
6. reversible delete storage and retention;
7. command semantics for inspecting and addressing history;
8. limits and policies for granular undo and redo.
