use std::fs::{File, OpenOptions};
use std::path::Path;

use anyhow::{Context, Result};

/// A simple file-based lock. The lock is released when dropped.
pub struct FileLock {
    _file: File,
    path: std::path::PathBuf,
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Acquire an exclusive lock by creating a lock file.
/// Retries briefly if the lock is held, then fails.
pub fn acquire_lock(path: &Path) -> Result<FileLock> {
    for attempt in 0..50 {
        match OpenOptions::new().write(true).create_new(true).open(path) {
            Ok(file) => {
                return Ok(FileLock {
                    _file: file,
                    path: path.to_path_buf(),
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                // Check if the lock is stale (older than 10 seconds)
                if let Ok(metadata) = std::fs::metadata(path)
                    && let Ok(modified) = metadata.modified()
                    && modified.elapsed().unwrap_or_default() > std::time::Duration::from_secs(10)
                {
                    // Stale lock - remove it and retry
                    let _ = std::fs::remove_file(path);
                    continue;
                }
                if attempt < 49 {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                    continue;
                }
                anyhow::bail!(
                    "could not acquire lock at {}. Another ark process may be running.",
                    path.display()
                );
            }
            Err(e) => {
                return Err(e)
                    .with_context(|| format!("failed to create lock file: {}", path.display()));
            }
        }
    }
    anyhow::bail!("could not acquire lock at {} after retries", path.display())
}
