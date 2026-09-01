# Research 004 — Filesystem identity and external drift

Status: initial design research  
Date: 2026-09-01

## Purpose

Define how dfman should identify filesystem objects strongly enough to support Basket persistence, operation journaling and granular undo in a filesystem that can also be modified by other actors.

The central constraint is simple:

> dfman is not the exclusive owner of the filesystem.

Any journal entry is historical evidence, not a guarantee that the current filesystem still matches that history.

## Identity layers

A pathname is a locator, not a durable identity.

For Windows filesystems that expose stable file identifiers, dfman should represent identity using the strongest inexpensive information available.

Candidate Windows identity:

```text
FileIdentity
  volumeSerial
  fileId128
```

Microsoft documents `FILE_ID_INFO` as containing both a volume serial number and a 128-bit file identifier, and states that their combination uniquely identifies a file on a single computer.

This is stronger than a path and allows dfman to distinguish:

- same path, same object;
- same path, replacement object;
- same object, renamed path;
- same object, moved path within a volume.

## Same-volume move / rename

A rename or move that remains inside the same filesystem volume can preserve object identity even when the path changes.

Conceptually:

```text
before
  identity = V1:F123
  path = C:\A\photo.jpg

after
  identity = V1:F123
  path = C:\B\renamed.jpg
```

This is the ideal case for journal reconciliation and granular undo.

The journal should therefore track both identity and observed path history.

## Cross-volume move

Cross-volume movement must be treated differently.

Windows may implement a cross-volume move as copy + delete. In that case the destination object is not the same filesystem object as the source and should receive a new identity.

Conceptually:

```text
source
  identity = V1:F123

copy to another volume

result
  identity = V2:F987
```

The journal may still record logical lineage:

```text
originIdentity      V1:F123
resultIdentity      V2:F987
relationship        copied_from / moved_across_volume
```

But dfman must not claim filesystem identity continuity where none exists.

Content hash and metadata can strengthen evidence that the result faithfully represents the source contents.

## Hard links

NTFS hard links introduce an important distinction between file object identity and pathname identity.

Multiple paths may refer to the same underlying file object.

Therefore:

```text
C:\A\x.dat -> V1:F555
C:\B\y.dat -> V1:F555
```

can both be true.

Consequences:

1. Basket entries must preserve the selected pathname even when multiple entries share one file identity.
2. Content changes through any hard link affect the same object.
3. Deleting one pathname does not necessarily delete the file object if another hard link remains.
4. Undo logic for delete must distinguish unlinking a pathname from destroying the final link to an object.

The journal should therefore retain both `identity` and `path binding` information.

## Replacement at the same pathname

This is a critical external-drift scenario.

Example:

```text
OP-100 creates C:\Work\report.dat

external actor:
  deletes report.dat
  creates a different report.dat
```

Path comparison alone would falsely conclude that the journaled object still exists.

Identity comparison should classify this as:

```text
replaced
```

If strong identity is unavailable, metadata and optional content hashing may reduce ambiguity, but must not be represented as certainty unless the evidence supports it.

## Content changes on the same object

An external process may modify an object without changing its identity.

Example:

```text
identity remains V1:F123
size / timestamp / content changes
```

This should not be classified as replacement. It is the same filesystem object with modified state.

For undo, this distinction matters:

```text
MOVE B -> A
```

may still be mechanically possible, but it is no longer an exact restoration of the historical content if the file changed after the original operation.

Recommended classification:

```text
same_object_content_changed
```

## Validation ladder

Validation should be proportional to the risk and cost of the operation.

### Level 0 — locator

```text
path exists?
```

Very cheap, weak evidence.

### Level 1 — filesystem identity

```text
volumeSerial + fileId
```

Strong evidence of object continuity where supported.

### Level 2 — cheap metadata fingerprint

Candidate fields:

```text
size
lastWriteTime
attributes
creation/change time when meaningful
reparse tag
```

Useful to detect state changes without reading file contents.

### Level 3 — content fingerprint

```text
SHA-256
```

Strong content-integrity evidence, but potentially expensive because all bytes must be read.

Hashing should therefore be lazy, policy-driven and reusable when an operation is already reading the content.

## Hash strategy

SHA-256 should not be mandatory for all Basket entries or all operations.

Recommended conceptual policies:

```text
FAST
  identity + cheap metadata

SAFE
  identity + metadata
  hash only on ambiguity / conflict

VERIFY
  hash source and/or result while data is already being processed
```

Names are provisional.

### Opportunistic hashing

If dfman is already reading all file contents, hashing can be computed with comparatively low additional I/O cost.

Examples:

```text
copy --verify
archive creation
explicit hash command
integrity check
```

The resulting digest should be persisted in the journal and reused later.

## Directories

Hashing directory contents as a single integrity check is not equivalent to hashing a file.

Directory validation must consider:

- directory object identity where available;
- child pathname set;
- child object identities;
- recursively changed contents only when required.

A full recursive Merkle-style directory fingerprint could exist later, but is not required for the initial architecture.

## Network filesystems / SMB

Windows exposes extended file information APIs over SMB 3.0, but dfman must not assume that every remote server, protocol version or underlying filesystem provides equally stable identity semantics.

The FileSource should advertise identity capabilities.

Conceptually:

```text
IdentityCapabilities
  strongObjectId
  volumeId
  hardLinks
  stableAcrossRename
  contentHash
```

If strong identity is unavailable, the journal must explicitly downgrade confidence instead of silently pretending path equality is sufficient.

## FAT / exFAT and weaker filesystems

The same capability-based rule applies.

A filesystem may expose weaker or different identifiers than NTFS/ReFS. dfman should not hard-code NTFS assumptions into the operation engine.

The local Windows FileSource can provide the strongest identity available for the mounted filesystem.

## Reconciliation states

A journal item compared with current reality should be classified rather than reduced to a Boolean match.

Candidate states:

```text
unchanged
moved_or_renamed
same_object_metadata_changed
same_object_content_changed
missing
replaced
ambiguous
destination_occupied
identity_unavailable
```

These states feed directly into operation and undo planning.

## Journal model extension

A journal item should conceptually support:

```text
JournalItem
  operationId
  sourcePathBefore
  pathAfter

  sourceIdentity?
  resultIdentity?

  metadataBefore
  metadataAfter

  contentHashBefore?
  contentHashAfter?

  relationship
  executionResult
```

The optional nature of strong identity and hashes must be explicit.

## Design principle

> Path identifies where an object was observed; identity identifies which object it was.

And:

> The journal records historical observations. Every recovery action must reconcile those observations with current filesystem reality before execution.

## Consequences for undo

Undo must never blindly replay inverse path operations.

It must first answer:

```text
Does the expected object still exist?
Is it still the same object?
Has its content changed?
Has another object occupied the historical destination?
Have later journaled operations transformed the same object?
```

Only then can the inverse plan be classified as safe, adjusted, conflicting or impossible.

## Follow-up

1. Define a platform-neutral `FileIdentity` abstraction.
2. Define capability negotiation on `FileSource`.
3. Prototype Windows identity retrieval through `FILE_ID_INFO`.
4. Verify identity behaviour empirically for rename, same-volume move, cross-volume move, hard links and replacement.
5. Define hash caching and invalidation policy.
6. Connect reconciliation states to granular undo planning.
