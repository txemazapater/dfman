use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchContext {
    pub current_path: PathBuf,
    pub left_path: Option<PathBuf>,
    pub right_path: Option<PathBuf>,
    pub selected_entries: Vec<PathBuf>,
}

impl LaunchContext {
    #[must_use]
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self {
            current_path: path.into(),
            left_path: None,
            right_path: None,
            selected_entries: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotEntry {
    pub path: PathBuf,
    pub is_dir: bool,
    pub len: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectorySnapshot {
    pub root: PathBuf,
    pub entries: Vec<SnapshotEntry>,
}

impl DirectorySnapshot {
    /// Builds a cheap, non-recursive snapshot using only metadata required by the MVP.
    ///
    /// More expensive metadata must be retrieved explicitly by later enrichment stages.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the directory cannot be enumerated or one of its entries
    /// cannot be read.
    pub fn scan(path: impl AsRef<Path>) -> io::Result<Self> {
        let root = path.as_ref().to_path_buf();
        let mut entries = Vec::new();

        for item in fs::read_dir(&root)? {
            let item = item?;
            let metadata = item.metadata()?;
            entries.push(SnapshotEntry {
                path: item.path(),
                is_dir: metadata.is_dir(),
                len: metadata.len(),
            });
        }

        Ok(Self { root, entries })
    }

    #[must_use]
    pub fn file_count(&self) -> usize {
        self.entries.iter().filter(|entry| !entry.is_dir).count()
    }

    #[must_use]
    pub fn directory_count(&self) -> usize {
        self.entries.iter().filter(|entry| entry.is_dir).count()
    }
}

#[cfg(test)]
mod tests {
    use super::{DirectorySnapshot, LaunchContext};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn launch_context_starts_without_optional_panel_or_selection_state() {
        let context = LaunchContext::at("example");

        assert_eq!(context.current_path, PathBuf::from("example"));
        assert!(context.left_path.is_none());
        assert!(context.right_path.is_none());
        assert!(context.selected_entries.is_empty());
    }

    #[test]
    fn scans_a_directory_without_recursing() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be valid")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("dfman-core-{unique}"));
        let child = root.join("child");

        fs::create_dir_all(&child).expect("temporary directory should be created");
        fs::write(root.join("one.txt"), b"one").expect("temporary file should be created");
        fs::write(child.join("two.txt"), b"two").expect("nested temporary file should be created");

        let snapshot = DirectorySnapshot::scan(&root).expect("snapshot should succeed");

        assert_eq!(snapshot.file_count(), 1);
        assert_eq!(snapshot.directory_count(), 1);
        assert_eq!(snapshot.entries.len(), 2);

        fs::remove_dir_all(root).expect("temporary directory should be removed");
    }
}
