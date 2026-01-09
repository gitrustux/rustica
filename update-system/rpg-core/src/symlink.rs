// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! Atomic symlink operations for safe version switching

use std::path::Path;

/// A symlink that can be atomically updated
#[derive(Debug, Clone)]
pub struct Symlink {
    /// The path where the symlink is located
    pub link_path: std::path::PathBuf,
}

impl Symlink {
    /// Create a new symlink
    pub fn new(link_path: impl AsRef<Path>) -> Self {
        Self {
            link_path: link_path.as_ref().to_path_buf(),
        }
    }

    /// Create a symlink pointing to the target
    pub fn create(&self, target: impl AsRef<Path>) -> crate::Result<()> {
        let target = target.as_ref();

        // Remove existing link if present
        if self.link_path.exists() {
            std::fs::remove_file(&self.link_path)?;
        }

        // Create parent directory if needed
        if let Some(parent) = self.link_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        // Create the symlink
        std::os::unix::fs::symlink(target, &self.link_path)?;

        Ok(())
    }

    /// Read the target of the symlink
    pub fn read(&self) -> crate::Result<std::path::PathBuf> {
        if !self.link_path.exists() {
            return Err(crate::Error::Layout(format!(
                "Symlink does not exist: {}",
                self.link_path.display()
            )));
        }

        self.link_path
            .read_link()
            .map_err(|e| crate::Error::Io(e))
    }

    /// Check if the symlink exists
    pub fn exists(&self) -> bool {
        self.link_path.exists() && self.link_path.is_symlink()
    }

    /// Remove the symlink
    pub fn remove(&self) -> crate::Result<()> {
        if self.exists() {
            std::fs::remove_file(&self.link_path)?;
        }
        Ok(())
    }
}

/// Atomically swap a symlink to point to a new target
///
/// This function ensures that the symlink update is atomic, preventing
/// any race conditions or inconsistent states.
///
/// # Arguments
///
/// * `link_path` - The path to the symlink
/// * `new_target` - The new target for the symlink
///
/// # Returns
///
/// `Ok(())` if the swap was successful, `Err` otherwise
pub fn atomic_symlink_swap(
    link_path: impl AsRef<Path>,
    new_target: impl AsRef<Path>,
) -> crate::Result<()> {
    let link_path = link_path.as_ref();
    let new_target = new_target.as_ref();

    // Verify the target exists
    if !new_target.exists() {
        return Err(crate::Error::Layout(format!(
            "Target does not exist: {}",
            new_target.display()
        )));
    }

    // Create a temporary symlink
    let temp_link = link_path.with_extension("tmp");

    // Remove temp link if it exists
    if temp_link.exists() {
        std::fs::remove_file(&temp_link)?;
    }

    // Create the temporary symlink
    std::os::unix::fs::symlink(new_target, &temp_link)?;

    // Atomic rename: on the same filesystem, this is atomic
    std::fs::rename(&temp_link, link_path)?;

    Ok(())
}

/// Atomically swap a symlink, storing the old target for rollback
///
/// # Arguments
///
/// * `link_path` - The path to the symlink
/// * `new_target` - The new target for the symlink
///
/// # Returns
///
/// The old target path, if there was one
pub fn atomic_symlink_swap_with_rollback(
    link_path: impl AsRef<Path>,
    new_target: impl AsRef<Path>,
) -> crate::Result<Option<std::path::PathBuf>> {
    let link_path = link_path.as_ref();

    // Read the old target if the link exists
    let old_target = if link_path.exists() && link_path.is_symlink() {
        Some(link_path.read_link()?)
    } else {
        None
    };

    // Perform the atomic swap
    atomic_symlink_swap(link_path, new_target)?;

    Ok(old_target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_symlink_create() {
        let temp_dir = TempDir::new().unwrap();
        let link_path = temp_dir.path().join("test-link");
        let target_path = temp_dir.path().join("target");

        // Create target directory
        std::fs::create_dir(&target_path).unwrap();

        // Create symlink
        let link = Symlink::new(&link_path);
        link.create(&target_path).unwrap();

        assert!(link.exists());
        assert_eq!(link.read().unwrap(), target_path);
    }

    #[test]
    fn test_atomic_symlink_swap() {
        let temp_dir = TempDir::new().unwrap();
        let link_path = temp_dir.path().join("current");
        let target1 = temp_dir.path().join("v1.0.0");
        let target2 = temp_dir.path().join("v2.0.0");

        // Create target directories
        std::fs::create_dir(&target1).unwrap();
        std::fs::create_dir(&target2).unwrap();

        // Initial symlink
        atomic_symlink_swap(&link_path, &target1).unwrap();
        assert_eq!(link_path.read_link().unwrap(), target1);

        // Atomic swap
        atomic_symlink_swap(&link_path, &target2).unwrap();
        assert_eq!(link_path.read_link().unwrap(), target2);
    }

    #[test]
    fn test_atomic_symlink_swap_with_rollback() {
        let temp_dir = TempDir::new().unwrap();
        let link_path = temp_dir.path().join("current");
        let target1 = temp_dir.path().join("v1.0.0");
        let target2 = temp_dir.path().join("v2.0.0");

        // Create target directories
        std::fs::create_dir(&target1).unwrap();
        std::fs::create_dir(&target2).unwrap();

        // Initial symlink
        atomic_symlink_swap(&link_path, &target1).unwrap();

        // Swap with rollback
        let old = atomic_symlink_swap_with_rollback(&link_path, &target2).unwrap();
        assert_eq!(old, Some(target1));
        assert_eq!(link_path.read_link().unwrap(), target2);
    }
}
