# ADR-001 — Basket, operation journal and granular undo

Status: Accepted  
Date: 2026-09-01

## Context

dfman is intended to operate primarily on subsets of filesystem entries rather than always on complete directory contents.

A central concept called the **Basket** represents the materialized set of entries that the user intends to manipulate. Selection is therefore treated as an operation that modifies the Basket rather than as transient UI state attached to panel rows.

Operations such as move, rename, delete, copy and similar actions act on a Basket and are first materialized as an `OperationPlan` before execution.

A simple last-operation undo would underuse the information already available in the Basket and OperationPlan. If execution is journaled at item level, the history can support substantially more precise recovery.

A further constraint is fundamental: **dfman is never assumed to be the exclusive owner of the filesystem**. Files and directories may be modified, renamed, replaced, deleted or recreated by other applications, services, users or machines between the original operation and any later inspection, retry or undo request.

Therefore journaled history describes what dfman observed and changed at a point in time; it is not proof that the current filesystem still matches that history.

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

### 4. Journal entries record identity and validation fingerprints

A path is location, not identity.

Each journaled item should retain the strongest practical identity and integrity information available at the time of the operation. The exact fields are platform dependent, but conceptually include three layers:

```text
Identity
  filesystem / volume identity
  file identity when supported

Cheap fingerprint
  path at that moment
  entry type
  size
  selected timestamps
  relevant attributes

Content fingerprint
  cryptographic hash when required or useful
```

On filesystems such as NTFS, a stable file identifier combined with volume identity can help distinguish a moved or renamed object from a different object later created at the same path.

A cryptographic hash such as SHA-256 can validate file content independently of path and most metadata. However, content hashing is an I/O operation proportional to file size and therefore must not become an unconditional cost of directory enumeration or every operation.

### 5. Validation is layered rather than binary

Before a destructive operation, retry, rollback or historical undo, dfman validates the involved entries against the current filesystem.

Validation should use progressively stronger checks only when necessary:

```text
Level 0 — existence / location
Level 1 — filesystem identity
Level 2 — cheap metadata fingerprint
Level 3 — content hash
```

A typical decision path may be:

```text
same volume + same file id
        |
        +-- metadata unchanged --> high confidence
        |
        +-- metadata changed ----> hash if content integrity matters

file id unavailable
        |
        +-- path + metadata match --> provisional match
        |
        +-- ambiguous ------------> hash / conflict
```

The implementation must not treat timestamps alone as proof of content identity.

### 6. Hashing is policy-driven and reusable

SHA-256 is the initial preferred content fingerprint candidate because it is widely available, collision resistant for this purpose and suitable for integrity validation.

Hash computation should be demand-driven. Candidate triggers include:

- the operation itself requires content verification;
- undo safety cannot be established from identity and metadata;
- a copy operation is configured for verified copy;
- the user explicitly requests hashing;
- an entry is about to be deleted by an inverse operation and must be proven to be the object previously created by dfman.

Once calculated, a hash may be retained in the journal and reused as long as the journal also records the state against which it was calculated.

A journal must never silently assume that an old hash describes current content merely because the path is unchanged.

### 7. Filesystem drift is an explicit state

The engine must model divergence between journaled history and current reality.

Useful validation outcomes include:

```text
unchanged
moved_or_renamed
metadata_changed
content_changed
missing
replaced
ambiguous
destination_occupied
identity_unavailable
```

For example, after:

```text
OP-100 COPY A -> B
```

an undo must not simply delete `B` because that pathname exists. It must establish that current `B` is still the object produced by `OP-100` or otherwise refuse / require explicit resolution.

Likewise, after:

```text
OP-100 MOVE A -> B
```

if another actor modifies the content at `B`, an inverse move back to `A` may still be technically possible but is no longer semantically equivalent to undoing the original operation. dfman must surface that distinction.

### 8. Undo is generated from journaled facts

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

### 9. History supports granular undo

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

### 10. Granular undo is dependency-aware

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

### 11. Undo itself is journaled

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
 validate current reality
    |
 inverse plan
    v
Undo operation
    |
 execute
    v
Journal entry
```

### 12. Reversible delete should be controlled by dfman

Where practical, the normal `delete` operation should use dfman-managed reversible storage or an equivalent mechanism that retains reliable original-location metadata.

A distinct explicitly irreversible operation, tentatively `purge`, may permanently destroy entries.

The exact storage strategy, retention policy and cross-volume behaviour will be defined separately.

## Consequences

### Positive

- The Basket becomes the common unit for selection, execution, retry and recovery.
- Partial failures can be understood and recovered without reconstructing history heuristically.
- Undo can operate at operation level or item level.
- Redo naturally follows from journaling inverse operations.
- External filesystem changes are detected rather than overwritten by historical assumptions.
- File identity and content integrity are treated as separate concerns.
- Cryptographic hashes provide strong validation when cheaper evidence is insufficient.
- The user can inspect what dfman actually changed rather than relying on opaque filesystem side effects.
- Batch operations become safer because their exact scope and outcomes remain addressable.

### Costs

- Persistent operation history becomes part of the core architecture rather than an optional UI feature.
- Entry identity must be stronger than path alone where the platform allows it.
- Non-LIFO undo requires dependency and conflict analysis.
- Journal consistency and crash recovery become important design concerns.
- Content hashing may be expensive for large files or large Baskets and therefore requires policy and caching.
- Reversible delete requires managed storage and retention rules.

## Architectural principles

> The Basket defines the scope of an operation; the journal records the facts of its execution; undo is a validated inverse operation over those facts.

> History in dfman is executable history, not merely a log.

> The journal records history; the current filesystem remains authoritative for present reality.

> A matching pathname is not sufficient evidence that an object is the same object.

## Follow-up decisions

Separate design work is required for:

1. filesystem entry identity across NTFS and other filesystems;
2. validation fingerprints and SHA policy;
3. Basket lifecycle and persistence;
4. operation and item state machines;
5. journal persistence and crash consistency;
6. dependency analysis between historical operations;
7. reversible delete storage and retention;
8. command semantics for inspecting and addressing history;
9. limits and policies for granular undo and redo;
10. conflict resolution when current filesystem state diverges from journal history.
