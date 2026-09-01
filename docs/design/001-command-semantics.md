# Design 001 — dfman command semantics

Status: exploratory  
Date: 2026-09-01

## Intent

dfman should not be limited to keyboard shortcuts layered over a dual-panel file manager.

A compact command semantics may become one of the project's defining features: a way to describe sets of filesystem entries and operations on those sets directly.

The goal is not to build a general-purpose shell or programming language. The goal is to make common file-management intentions concise, composable and predictable.

## Core mental model

The command layer works primarily against the current directory snapshots.

```text
source set -> predicates -> resulting set -> operation
```

Examples:

```text
select *.jpg
select size > 10MB
select ext in jpg,png,webp
select modified < 2025-01-01
copy selected right
move selected D:\Archive
```

The same semantics should be usable interactively and, eventually, non-interactively.

## Objects with useful names

Candidate built-in set references:

```text
current      current item
selected     current selection
all          all entries in the active snapshot
files        all files
folders      all directories
left         left panel snapshot / location
right        right panel snapshot / location
```

Potential persistent named sets:

```text
set largephotos = select ext in jpg,jpeg and size > 20MB
copy @largephotos right
```

Named sets should initially refer to entry identities from a particular snapshot generation, not magical permanently live queries.

## Candidate predicates

Cheap predicates based on enumeration metadata:

```text
name
ext
size
created
modified
accessed
attributes
kind
```

Examples:

```text
select ext = jpg
select size >= 100MB
select kind = file
select modified >= 2026-08-01
select name like "IMG_*"
select ext in jpg,png and size > 5MB
```

Predicates requiring enrichment can exist but must declare their cost:

```text
owner
links
streams
hash
mime
```

The execution engine can therefore know whether a command is snapshot-only or requires additional filesystem work.

## Verbs

The language should prefer a small set of obvious verbs:

```text
select
unselect
copy
move
rename
delete
mkdir
compare
refresh
show
sort
filter
```

Future verbs may include:

```text
sync
hash
dupe
archive
extract
```

No verb should be introduced merely to expose an internal implementation detail.

## Direction and destination

The dual-panel model gives us convenient semantic destinations:

```text
copy selected right
move selected left
```

Explicit paths remain valid:

```text
copy selected D:\Archive\Photos
```

This is deliberately more expressive than assigning every meaningful operation to a function key, while function keys can remain shortcuts for the common cases.

## Selection as a first-class operation

Classic file managers treat selection mostly as transient UI state. dfman can go further and make a selection a meaningful set.

For example:

```text
select ext = jpg
select + ext = png
select - size < 100KB
```

Possible compact set algebra, if it remains readable:

```text
+   union / add
-   subtract
&   intersection
```

This notation is exploratory; readability is more important than clever syntax.

## Pipeline syntax: deliberately undecided

A Unix-like pipeline is tempting:

```text
all | where ext=jpg | where size>10MB | copy right
```

But it risks making dfman look like a generic shell and may become visually noisy.

A more domain-specific form may be clearer:

```text
copy all where ext=jpg and size>10MB to right
```

or:

```text
select ext=jpg and size>10MB
copy selected right
```

The parser should therefore be designed only after experimenting with real workflows.

## Two important distinctions

### Query versus action

```text
select size > 1GB
```

changes only dfman's selection state.

```text
delete selected
```

changes the filesystem.

The grammar and UI should make destructive actions unmistakable.

### Snapshot operation versus live filesystem operation

Queries run against a known snapshot generation.

Before a destructive operation starts, the operation engine must validate that the source entries still exist and resolve any relevant mismatch between snapshot state and current filesystem state.

This preserves the snapshot philosophy without pretending the filesystem has remained frozen.

## Possible dry-run semantics

Because commands are declarative enough, dfman could eventually support:

```text
preview move selected right
```

or a generic modifier:

```text
move selected right --dry-run
```

The important idea is that the command can first be compiled into an operation plan before execution.

```text
command
  -> parse
  -> resolve set
  -> determine required metadata
  -> validate
  -> operation plan
  -> execute
```

This is potentially a major architectural advantage of having our own semantics.

## Why this may differentiate dfman

FAR and Double Commander are primarily file managers with commands attached to UI actions.

dfman can instead treat the panel as a visual representation of sets and locations, while the command language expresses intent over those sets.

In other words:

> The panels show the filesystem; the semantics operate on it.

This makes the UI and command line two front ends to the same operation model rather than separate ways of implementing file actions.

## Scope guard

Do not build a shell language prematurely.

The first semantic prototype only needs enough grammar to prove three things:

1. selecting sets is faster than manual marking for realistic workloads;
2. the same selection can feed normal file operations;
3. metadata requirements can be planned before I/O is performed.

A candidate minimal grammar is therefore:

```text
select <predicate>
unselect <predicate>
copy <set> <destination>
move <set> <destination>
delete <set>
mkdir <name>
refresh
```

with predicates initially limited to:

```text
name, ext, size, kind, modified
```

That is enough to discover whether the idea deserves to become a core feature.
