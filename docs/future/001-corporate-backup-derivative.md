# Future 001 — Corporate backup derivative

Status: deferred / out of scope for current dfman development  
Date: 2026-09-02

## Context

Initial dfman benchmarks show that the current unoptimized Rust implementation can enumerate and materialize directory snapshots very quickly. More importantly, several architectural concepts already being designed for dfman are also directly useful in a backup engine:

- filesystem snapshots;
- explicit sets / Basket;
- operation planning;
- validation and hashing;
- journaling;
- historical object identity;
- granular retry and recovery;
- declarative command semantics.

This creates a credible future path toward a corporate backup product or service derived from shared dfman infrastructure.

## Important scope boundary

This is **not** part of dfman's current scope.

The dfman project remains focused on building a fast, predictable, terminal-oriented file operations manager. Backup-specific concerns must not be introduced into the current core merely because they may be useful later.

The purpose of this note is to preserve the opportunity without allowing it to derail dfman.

## Potential future architecture

A future backup application should consume reusable components rather than turning dfman itself into a backup product.

```text
                 shared filesystem core
                         |
            +------------+------------+
            |                         |
          dfman                    dfbackup
   interactive file ops       corporate backup
```

Candidate shared components may eventually include:

```text
dfman-core
filesystem identity
snapshot model
Basket / object sets
hash / validation services
operation planning primitives
journal infrastructure
```

Backup-specific capabilities should remain separate:

```text
VSS integration
backup repositories
retention policies
content-addressable storage
deduplication
compression
encryption
remote transport
scheduling
restore catalogs
backup consistency policies
```

## Candidate backup flow

A future Windows-oriented backup engine could use a model such as:

```text
live filesystem
      |
      v
VSS snapshot
      |
      v
DirectorySnapshot
      |
      v
changed-object query
      |
      v
Basket
      |
      v
BackupPlan
      |
      v
hash / verify / store
      |
      v
Backup Journal + Manifest
```

The Basket maps naturally to an incremental backup set: the materialized set of objects selected for a particular backup operation.

## Content integrity and deduplication

SHA-256 or another strong content hash could serve both integrity validation and future content-addressable storage.

Conceptually:

```text
object content
    |
    v
SHA-256
    |
    +--> integrity evidence
    |
    +--> deduplication key
    |
    +--> content-addressable object id
```

A backup manifest could then map historical filesystem paths to immutable stored objects rather than duplicating unchanged content for every backup generation.

## Declarative semantics

The dfman declarative model could also inspire a backup-specific language without coupling the two applications.

Examples:

```text
BACKUP FROM \\SRVFILES\Engineering
WHERE changed_since = last_backup
TO repository('nas01')
VERIFY SHA256
```

or through a natural-intent layer:

```text
Back up the engineering shares to the NAS and keep 30 days.
```

As with dfman, any natural-language layer would only translate intent into a validated formal plan; it would never manipulate backup storage directly.

## Decision

Preserve the possibility of a future backup derivative, but make no backup-specific implementation changes in dfman at this stage.

The current priority remains:

1. prove dfman's snapshot / Basket / planner / journal model;
2. build a reliable file-operation engine;
3. implement the terminal interaction model;
4. only then evaluate which core pieces deserve extraction as reusable libraries.

## Guiding principle

> Do not build a backup product inside dfman. Build dfman well enough that a future backup product can reuse its proven primitives.
