# Design 002 — SQL-inspired command semantics

Status: exploratory  
Date: 2026-09-01

## Intent

dfman should support a compact declarative command language for expressing operations over filesystem subsets.

The language should be inspired by SQL in its mental model:

- describe the set;
- describe the predicate;
- describe the transformation;
- let the engine plan and execute.

The goal is not to embed SQL or create a general programming language.

## Why SQL is a good influence

Filesystem work is naturally set-oriented.

Examples:

```text
all entries in the current snapshot
all JPG files larger than 10 MB
all files modified before a date
all Basket entries whose content has changed
all journal items from operation OP-1042 that completed successfully
```

These are queries over sets.

SQL provides useful ideas:

```text
FROM    -> source set
WHERE   -> predicate
SELECT  -> projection / basket materialization
ORDER BY-> presentation ordering
UPDATE  -> transformation
DELETE  -> destructive operation
```

But dfman should use filesystem-native verbs and concepts rather than imitate SQL syntax blindly.

## Proposed conceptual sources

The language can operate on named domain sets:

```text
current     current panel snapshot
left        left panel snapshot
right       right panel snapshot
basket      current Basket
history     operation journal
operation X entries belonging to a journal operation
```

Potential future sources:

```text
trash
search results
named baskets
```

## Basket materialization

A SQL-inspired selection could look like:

```text
select from current where ext = 'jpg'
```

Meaning:

```text
query current snapshot
materialize matching entries into Basket
```

Useful variants:

```text
select add from current where ext = 'png'
select remove from basket where size < 100KB
select clear
```

The exact grammar remains open.

A shorter interactive form may also be supported:

```text
select where ext = 'jpg'
```

with `from current` implied.

## Predicates

Candidate cheap predicates:

```text
name
ext
size
modified
created
accessed
attributes
is_file
is_dir
is_link
```

Examples:

```text
select where ext in ('jpg','png','webp')
select where size > 10MB
select where modified < '2025-01-01'
select where is_file and size between 1MB and 10MB
```

Potential expensive predicates:

```text
owner
mime
hash
content
```

The query planner should detect required metadata before execution.

Example:

```text
select where size > 10MB
```

requires only snapshot metadata.

```text
select where owner = 'txema'
```

requires owner enrichment.

```text
select where sha256 = '...'
```

requires content hashing.

Thus the language naturally feeds the operation planner.

## Filesystem operations as set transformations

Operations should read naturally as transformations over the Basket.

Examples:

```text
move basket to right
copy basket to 'D:\Archive'
delete basket
rename basket using ...
hash basket
```

A more SQL-like alternative is possible:

```text
move to right where ext = 'jpg'
```

which the engine could internally decompose into:

```text
SELECT -> Basket
PLAN
MOVE
```

However, preserving an explicit Basket phase may be safer because the user can inspect the exact scope before execution.

## Two-stage workflow as default

Recommended default interaction:

```text
select where ext = 'jpg' and size > 10MB
basket
move to right
```

This preserves the important dfman model:

```text
query -> materialized Basket -> operation plan -> execution
```

## One-shot statements

For experienced users, one-shot commands could be valid:

```text
move to right where ext = 'jpg' and size > 10MB
```

The engine should still internally create a temporary Basket and OperationPlan.

Thus syntax convenience does not bypass safety architecture.

## History as a queryable set

The operation journal is also naturally relational.

Examples:

```text
select from history where type = 'MOVE'
select from operation 'OP-1042' where result = 'completed'
undo operation 'OP-1042' where result = 'completed'
```

This is especially attractive for granular undo.

Potential examples:

```text
undo OP-1042 where path like '*.jpg'
undo OP-1042 where drift = 'unchanged'
retry OP-1042 where result = 'failed'
```

Again, these would compile to materialized subsets and validated plans rather than execute as raw journal replay.

## Inspect / explain

SQL's `EXPLAIN` concept is particularly relevant to dfman.

Potential command:

```text
explain select where owner = 'txema' and size > 1GB
```

could report:

```text
Source: current snapshot
Candidates: 18,442
Cheap filter size > 1GB: 37
Owner metadata required for 37 entries
Estimated expensive metadata reads: 37
No content reads required
```

Likewise:

```text
explain move to right where ext = 'jpg'
```

could describe the generated OperationPlan without executing it.

This may become one of the strongest usability features of the semantic layer.

## Ordering and display vs membership

`ORDER BY`-style semantics should affect presentation, not Basket identity.

Example:

```text
show basket order by size desc
```

must not change Basket membership.

This reinforces the separation between:

```text
set semantics
presentation semantics
```

## Proposed minimal grammar shape

Not final syntax, only direction:

```text
SELECT [ADD|REMOVE] [FROM source] [WHERE predicate]
SHOW source [WHERE predicate] [ORDER BY field]
MOVE [source] TO destination [WHERE predicate]
COPY [source] TO destination [WHERE predicate]
DELETE [source] [WHERE predicate]
UNDO operation [WHERE predicate]
RETRY operation [WHERE predicate]
EXPLAIN statement
```

Interactive aliases may remain shorter and case-insensitive.

## What to avoid

The semantic layer should not grow into:

```text
variables
loops
functions
procedures
arbitrary expressions with side effects
general-purpose scripting
```

For those needs, dfman should expose a CLI/API that real scripting languages can call.

## Architectural consequence

Commands should compile into domain objects rather than directly touching the filesystem.

Conceptually:

```text
statement
   |
 parser
   v
query / command AST
   |
 planner
   +--> metadata requirements
   +--> source set
   +--> Basket materialization
   v
OperationPlan
   |
 validator
   v
execution engine
```

This makes semantic commands, keyboard actions and any future automation front-end converge on the same operation engine.

## Principle

> dfman commands describe intent over sets; they do not directly manipulate panel rows.

And:

> SQL is an inspiration for declarative set semantics, not a compatibility target.
