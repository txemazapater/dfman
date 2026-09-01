# Research 002 — Directory enumeration and refresh strategy

Status: initial investigation  
Date: 2026-09-01

## Purpose

Study how FAR Manager and Double Commander enumerate directories, detect external filesystem changes and refresh their panels.

This area is strategically important for dfman because one of the explicit motivations for the project is avoiding the disruptive and sometimes expensive continuous-refresh behaviour found in modern graphical file explorers.

## FAR Manager — directory enumeration

FAR does not simply wrap the classic `FindFirstFile` / `FindNextFile` path for its main enumeration implementation.

Its filesystem layer opens the directory with `FILE_LIST_DIRECTORY` and enumerates entries through `NtQueryDirectoryFile`, using a 64 KiB buffer. It first attempts `FileIdBothDirectoryInformation` and falls back to `FileBothDirectoryInformation` when required.

The implementation reads batches of directory entries into memory and walks `NextEntryOffset` inside those buffers. It also performs a second read eagerly after the first successful query to work around a historical issue and reduce latency.

Conceptually:

```text
open directory
    |
NtQueryDirectoryFile(64 KiB buffer)
    |
multiple entries returned per syscall
    |
walk entries in memory
    |
request next buffer only when needed
```

This is significantly more interesting for large directories than treating enumeration as one high-level call per visible file.

### Important lesson for dfman

On Windows, directory enumeration deserves its own low-level backend and should be benchmarked independently from the terminal UI.

The first prototype should compare at least:

- native/batched Windows enumeration similar to FAR;
- the normal runtime/library directory iterator offered by the chosen language;
- cost of retrieving only essential metadata versus richer metadata.

We should not assume that the most convenient standard-library iterator is fast enough for the project's main use case.

## FAR Manager — filesystem watching

FAR has an explicit `FileSystemWatcher` implementation based on `ReadDirectoryChangesW` with overlapped I/O.

It watches:

- file names;
- directory names;
- attributes;
- size;
- last-write time;
- last-access time;
- creation time;
- security changes.

However, an especially relevant design choice appears in the background watcher: FAR explicitly notes that filesystem changes can occur at a high rate and that it does not care about individual events. The watcher therefore waits so that bursts collapse to at most approximately one notification per second.

That means FAR already treats filesystem notifications as **invalidation signals**, not as a precious event stream that must be mirrored one-by-one into the panel.

This distinction is important.

```text
filesystem events
   x x x x x x x x x
            |
            v
      coalesced signal
            |
            v
       panel is stale
```

FAR also handles `ERROR_NOTIFY_ENUM_DIR`, which indicates that Windows could not retain all directory changes. Since FAR does not need individual events, it effectively treats this as a valid "directory changed" result and relies on re-enumeration.

### FAR also exposes a refresh hazard

`FileList` contains explicit machinery to prevent background re-reading from invalidating the list while code is iterating over it.

The source comments describe the exact failure scenario:

1. a lengthy operation is iterating through panel data;
2. an external filesystem change occurs;
3. a notification requests a refresh;
4. the panel re-reads the directory;
5. the original iteration resumes with references to data that no longer exists.

FAR therefore locks panel list data while it is in use and postpones updates when necessary.

This is a strong warning for dfman: **automatic refresh is not free architectural convenience**. It introduces lifetime, concurrency and UI stability problems even in a mature application.

## Double Commander — directory enumeration

Double Commander models listing as another operation: `TFileSystemListOperation` derives from the generic file-source operation model.

For the local filesystem it currently uses `FindFirstEx` / `FindNextEx`, creates `TFile` objects for the discovered entries and stores them in a `TFiles` collection.

The operation periodically calls `CheckOperationState`, so enumeration participates in the same operation lifecycle as other actions and can react to cancellation/state changes.

Conceptually:

```text
FileSource
   |
CreateListOperation(path)
   |
ListOperation
   |
FindFirstEx / FindNextEx
   |
TFiles snapshot/result
```

This separation is architecturally attractive even if FAR's lower-level Windows enumeration may ultimately be more interesting for performance.

### Lesson for dfman

The **operation boundary from Double Commander** and the **enumeration technique from FAR** are independent choices and can be combined.

For example:

```text
DirectoryListOperation
        |
        +-- Windows backend -> batched native enumeration
        +-- POSIX backend   -> platform-specific iterator
        |
        v
DirectorySnapshot
```

## Double Commander — watching

Double Commander has a generic `TFileSourceWatcher` abstraction with explicit event types such as:

- created;
- changed;
- deleted;
- renamed;
- watched object deleted;
- unknown change.

The local filesystem source adapts that interface to `TFileSystemWatcher`.

This is another useful architectural separation: watching is a **capability of a file source**, not a concern hardcoded into a particular panel.

For dfman, however, we probably do not need the UI to consume these fine-grained events directly.

## Proposed dfman strategy

The current recommendation is deliberately different from both Explorer-style continuous synchronization and a completely blind static panel.

### 1. A panel displays a `DirectorySnapshot`

A successful enumeration produces an immutable or logically immutable snapshot:

```text
DirectorySnapshot
  path
  generation
  captured_at
  entries[]
```

The panel renders that snapshot. Scrolling, sorting, selection and navigation operate on stable in-memory data.

### 2. Filesystem notifications mark the snapshot `dirty`

A watcher may exist, but its primary job is not to mutate the panel incrementally.

```text
ReadDirectoryChangesW
        |
    coalesce
        |
        v
snapshot.dirty = true
```

This keeps external-change awareness without allowing every filesystem event to redraw or reconstruct the panel.

### 3. Dirty does not necessarily mean immediate refresh

Possible policy:

```text
clean snapshot
     |
external change
     v
DIRTY
     |
     +-- explicit refresh
     +-- directory re-entry
     +-- safe idle point
     +-- before an operation that requires fresh state
```

The UI can show a tiny non-intrusive stale indicator rather than changing underneath the user.

Example:

```text
C:\Photos\Incoming                              [changed]
```

No cursor jump. No selection loss. No surprise resorting.

### 4. Operations initiated by dfman update state deliberately

When dfman itself completes an operation, we know what happened.

For an MVP, the safest behaviour is still to invalidate and re-enumerate the affected directory at a controlled point rather than attempting clever incremental mutation.

Later, if benchmarks justify it, known internal operations could produce a new snapshot more cheaply.

### 5. Panel data should not be modified in place by a watcher thread

The watcher communicates only an invalidation/state signal.

A new snapshot is built separately and then atomically replaces the old one when the UI decides it is appropriate.

```text
             old snapshot  <--- panel is reading this
                  |
filesystem ------>| dirty
                  |
           build new snapshot
                  |
             atomic swap
                  v
             new snapshot
```

This avoids the class of lifetime hazard documented in FAR's `FileList` implementation.

## Preliminary design principle

> Filesystem notifications tell dfman that its knowledge may be stale. They do not tell the UI what it must redraw.

And a second principle:

> A directory listing is a snapshot, not a live view of the filesystem.

This is probably one of the defining behavioural differences between dfman and a conventional graphical file explorer.

## Performance questions to benchmark

Before selecting implementation technology, build a small enumeration benchmark against representative directories:

- 1,000 files;
- 10,000 files;
- 100,000 files;
- mixed files/directories;
- local NVMe;
- nearly full filesystem;
- SMB/network path if relevant.

Measure separately:

1. raw enumeration time;
2. metadata extraction time;
3. allocation/object creation cost;
4. sort time;
5. terminal render time.

The benchmark must make it possible to distinguish a slow filesystem scan from a slow UI.

## Current recommendation

For the first dfman prototype:

```text
FileSource
    |
DirectoryListOperation
    |
platform enumeration backend
    |
DirectorySnapshot
    |
Panel

FileSourceWatcher
    |
coalesced invalidation
    |
DirectorySnapshot: dirty
```

Do **not** implement live per-event panel mutation.

Do **not** automatically rebuild a panel for every filesystem notification.

Do **not** let a watcher thread own panel state.

The next investigation should focus on the exact metadata collected during enumeration and on the cost of sorting/rendering very large snapshots. That will tell us where the real performance budget goes before choosing the final language and terminal framework.
