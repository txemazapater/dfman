# Research 003 — Metadata cost model

Status: initial design note  
Date: 2026-09-01

## Purpose

Define which file properties dfman should obtain during directory enumeration and which properties should be loaded only when a view, filter, command or operation actually requires them.

The objective is simple: directory listing must stay cheap and predictable even when a directory contains many thousands of entries.

## Observation from the reference implementations

FAR Manager already treats several properties lazily. Its panel item stores the basic find-data returned by enumeration, while properties such as allocation size, number of hard links, stream count, stream size, owner and content data are obtained only when requested or when the corresponding column is displayed.

Double Commander follows a similar idea through `RetrieveProperties`: the initial file object can contain the properties returned during enumeration and additional properties are retrieved only when requested.

This should become an explicit dfman design rule rather than an implementation detail.

## Proposed property tiers

### Tier 0 — identity

Always available without additional filesystem queries:

```text
name
path / parent identity
entry kind (file / directory / reparse-like entry)
```

### Tier 1 — enumeration metadata

Properties obtained as part of the native directory enumeration call and therefore considered cheap:

```text
attributes
type flags
size
creation time
last write time
last access time
change time (when available)
allocation size (only if already returned reliably)
file id (when cheaply available)
reparse tag
```

These values form the immutable directory snapshot.

### Tier 2 — explicit enrichment

Properties that may require one additional filesystem query per file or other non-trivial work:

```text
owner
ACL/security descriptor
hard-link count
streams
compressed/allocation size when not known
resolved symbolic-link target
MIME/type description
extended attributes
```

These must not be fetched merely because an entry is visible in a panel.

### Tier 3 — content-derived metadata

Potentially expensive information requiring file contents to be opened or processed:

```text
hash
media dimensions / duration
EXIF
text encoding
content signature
thumbnail
content search data
```

These are operations, not listing metadata.

## Proposed snapshot model

A panel snapshot should contain only Tier 0 and Tier 1 information by default.

```text
DirectorySnapshot
  path
  generation
  state: clean | dirty
  entries[]
    identity
    enumeration metadata
    enrichment cache (optional)
```

Enrichment does not mutate the meaning of the snapshot generation. It merely attaches cached information to entries already identified by the snapshot.

If the underlying directory changes, the snapshot becomes `dirty`; it is not automatically reconstructed.

## Semantic integration

The command/query engine can declare its property requirements.

Example:

```text
select size > 10MB
```

requires only Tier 1 and can execute immediately against the current snapshot.

```text
select owner = "txema"
```

requires Tier 2. The query planner may therefore:

1. determine which candidate entries need `owner`;
2. enrich only those entries;
3. evaluate the predicate;
4. cache the result for the lifetime of the snapshot.

A content query such as:

```text
select hash = @current.hash
```

would be a Tier 3 operation and should be visibly treated as such rather than silently turning a simple selection into a long filesystem scan.

## Important principle

> Display is not permission to perform I/O.

Merely showing an entry in a panel must not trigger arbitrary property retrieval, shell extensions, thumbnail generation or content inspection.

## Consequence for the terminal UI

Columns should have declared cost.

A user may request an expensive column explicitly, but dfman should know that enabling `owner`, `hash`, etc. is qualitatively different from showing `size` or `mtime`.

A future UI could indicate this distinction, but the underlying model should exist from the beginning.

## Initial recommendation

For the first Windows prototype, enumerate using the richest low-cost native directory record available and preserve the returned values directly. Do not issue secondary per-entry filesystem calls during normal listing.

This is more important than reproducing every column offered by FAR or Double Commander.
