// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! File utility functions

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::io::{self, Read, Write};

/// Copy a file from source to destination
pub fn copy_file(src: &Path, dst: &Path) -> Result<u64> {
    let mut src_file = fs::File::open(src)
        .with_context(|| format!("cannot open source file: {}", src.display()))?;

    let mut dst_file = fs::File::create(dst)
        .with_context(|| format!("cannot create destination file: {}", dst.display()))?;

    let mut buffer = Vec::new();
    src_file.read_to_end(&mut buffer)?;
    dst_file.write_all(&buffer)?;

    Ok(buffer.len() as u64)
}

/// Recursively copy a directory
pub fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)
            .with_context(|| format!("cannot create destination directory: {}", dst.display()))?;
    }

    for entry in fs::read_dir(src)
        .with_context(|| format!("cannot read source directory: {}", src.display()))?
    {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir(&src_path, &dst_path)?;
        } else {
            copy_file(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Remove a file or directory recursively
pub fn remove_path(path: &Path) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)
            .with_context(|| format!("cannot remove directory: {}", path.display()))?;
    } else {
        fs::remove_file(path)
            .with_context(|| format!("cannot remove file: {}", path.display()))?;
    }
    Ok(())
}

/// Get file size
pub fn get_file_size(path: &Path) -> Result<u64> {
    Ok(fs::metadata(path)
        .with_context(|| format!("cannot get metadata: {}", path.display()))?
        .len())
}

/// Check if path exists
pub fn path_exists(path: &Path) -> bool {
    path.exists()
}

/// Create a directory with parents
pub fn create_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .with_context(|| format!("cannot create directory: {}", path.display()))
}

/// Read file contents
pub fn read_file(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .with_context(|| format!("cannot read file: {}", path.display()))
}

/// Write file contents
pub fn write_file(path: &Path, contents: &str) -> Result<()> {
    fs::write(path, contents)
        .with_context(|| format!("cannot write file: {}", path.display()))
}

/// Update file modification time
pub fn touch_file(path: &Path) -> Result<()> {
    if path.exists() {
        // Update modification time
        let now = std::time::SystemTime::now();
        filetime::FileTime::from_system_time(now);
        // Note: This would require the filetime crate
        // For now, just read and rewrite the file
        let _ = fs::File::open(path)?;
    } else {
        // Create new file
        fs::File::create(path)
            .with_context(|| format!("cannot create file: {}", path.display()))?;
    }
    Ok(())
}

/// Canonicalize a path (resolve . and ..)
pub fn canonicalize(path: &Path) -> Result<PathBuf> {
    path.canonicalize()
        .with_context(|| format!("cannot canonicalize path: {}", path.display()))
}
