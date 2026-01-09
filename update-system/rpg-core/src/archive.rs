// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! RPG package archive format
//!
//! Packages are tar.gz archives with the following structure:
//! ```
//! package.rpg
//! ├── metadata.json          # Package metadata
//! ├── files/                 # Actual files to install
//! │   ├── usr/
//! │   ├── bin/
//! │   └── ...
//! ├── scripts/               # Installation scripts (optional)
//! │   ├── pre-install.sh
//! │   ├── post-install.sh
//! │   └── pre-remove.sh
//! └── signature.sig          # Detached signature (optional)
//! ```

use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use crate::package::{PackageKind, PackageMetadata};
use crate::signature::PackageSignature;
use crate::version::Version;

/// Package archive
#[derive(Debug, Clone)]
pub struct PackageArchive {
    /// Path to the archive file
    pub path: PathBuf,
    /// Package metadata
    pub metadata: PackageMetadata,
}

/// Package manifest (metadata.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    /// Package name
    pub name: String,

    /// Package version
    pub version: String,

    /// Package description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Package kind
    #[serde(rename = "type")]
    pub kind: String,

    /// Architecture (x86_64, aarch64, riscv64)
    pub arch: String,

    /// Dependencies
    #[serde(default)]
    pub dependencies: Vec<String>,

    /// Conflicts with
    #[serde(default)]
    pub conflicts: Vec<String>,

    /// Package size in bytes
    pub size: u64,

    /// SHA-256 checksum
    pub sha256: String,

    /// Download URL
    pub url: String,

    /// Maintainer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maintainer: Option<String>,

    /// Homepage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,

    /// License
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Installation size (uncompressed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed_size: Option<u64>,

    /// Files list
    #[serde(default)]
    pub files: Vec<String>,

    /// Directories to create
    #[serde(default)]
    pub directories: Vec<String>,

    /// Pre-install script
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_install: Option<String>,

    /// Post-install script
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_install: Option<String>,

    /// Pre-remove script
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_remove: Option<String>,

    /// Signature (base64)
    pub signature: String,
}

impl PackageManifest {
    /// Create a new manifest
    pub fn new(
        name: String,
        version: String,
        kind: PackageKind,
        arch: String,
        size: u64,
        sha256: String,
        url: String,
        signature: PackageSignature,
    ) -> Self {
        Self {
            name,
            version,
            description: None,
            kind: kind.as_str().to_string(),
            arch,
            dependencies: Vec::new(),
            conflicts: Vec::new(),
            size,
            sha256,
            url,
            maintainer: None,
            homepage: None,
            license: None,
            installed_size: None,
            files: Vec::new(),
            directories: Vec::new(),
            pre_install: None,
            post_install: None,
            pre_remove: None,
            signature: signature.to_base64(),
        }
    }

    /// Convert to PackageMetadata
    pub fn to_metadata(&self) -> crate::Result<PackageMetadata> {
        use crate::signature::PackageSignature;

        let version = Version::parse(&self.version)?;
        let kind = PackageKind::from_str(&self.kind)?;
        let signature = PackageSignature::from_base64(&self.signature)?;

        Ok(PackageMetadata::new(
            self.name.clone(),
            version,
            kind,
            self.size,
            self.sha256.clone(),
            signature,
            self.url.clone(),
        ))
    }
}

impl PackageArchive {
    /// Open an existing package archive
    pub fn open(path: impl AsRef<Path>) -> crate::Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(crate::Error::PackageNotFound(path.display().to_string()));
        }

        // Extract and read metadata
        let metadata = Self::extract_metadata(path)?;

        Ok(Self {
            path: path.to_path_buf(),
            metadata,
        })
    }

    /// Create a new package archive
    pub fn create(
        path: impl AsRef<Path>,
        manifest: PackageManifest,
        files: &[PathBuf],
    ) -> crate::Result<Self> {
        let path = path.as_ref();

        // Create temporary directory for staging
        let temp_dir = TempDir::new()?;
        let staging_dir = temp_dir.path();

        // Write manifest
        let manifest_path = staging_dir.join("metadata.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| crate::Error::Serialization(e.to_string()))?;
        fs::write(&manifest_path, manifest_json)?;

        // Copy files
        let files_dir = staging_dir.join("files");
        fs::create_dir_all(&files_dir)?;

        for file_path in files {
            if file_path.is_file() {
                let dest = files_dir.join(
                    file_path
                        .strip_prefix("/")
                        .unwrap_or(file_path),
                );
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(file_path, dest)?;
            }
        }

        // Create tar.gz archive
        let tar_gz = File::create(path)?;
        let enc = flate2::write::GzEncoder::new(tar_gz, flate2::Compression::default());
        let mut tar = tar::Builder::new(enc);

        tar.append_dir_all(".", staging_dir)?;

        // Finish the archive
        let enc = tar.into_inner()?;
        enc.finish()?;

        // Read back the metadata
        let metadata = manifest.to_metadata()?;

        Ok(Self {
            path: path.to_path_buf(),
            metadata,
        })
    }

    /// Extract metadata from package
    fn extract_metadata(path: &Path) -> crate::Result<PackageMetadata> {
        let file = File::open(path)?;
        let buf_reader = BufReader::new(file);
        let decoder = flate2::read::GzDecoder::new(buf_reader);
        let mut tar_archive = tar::Archive::new(decoder);

        // Find metadata.json
        for entry in tar_archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;

            if path.ends_with("metadata.json") {
                let mut contents = String::new();
                entry.read_to_string(&mut contents)?;
                let manifest: PackageManifest = serde_json::from_str(&contents)
                    .map_err(|e| crate::Error::Serialization(e.to_string()))?;
                return manifest.to_metadata();
            }
        }

        Err(crate::Error::Other(
            "metadata.json not found in package".to_string(),
        ))
    }

    /// Extract package to a directory
    pub fn extract(&self, dest: impl AsRef<Path>) -> crate::Result<()> {
        let dest = dest.as_ref();

        // Create destination directory
        fs::create_dir_all(dest)?;

        let file = File::open(&self.path)?;
        let buf_reader = BufReader::new(file);
        let decoder = flate2::read::GzDecoder::new(buf_reader);
        let mut tar_archive = tar::Archive::new(decoder);

        tar_archive.unpack(dest)?;

        Ok(())
    }

    /// Extract files to a directory
    pub fn extract_files(&self, dest: impl AsRef<Path>) -> crate::Result<()> {
        let dest = dest.as_ref();

        // Extract to temp directory first
        let temp_dir = TempDir::new()?;
        self.extract(temp_dir.path())?;

        // Move files/ directory to destination
        let files_source = temp_dir.path().join("files");
        if files_source.exists() {
            for entry in fs::read_dir(files_source)? {
                let entry = entry?;
                let target = dest.join(entry.file_name());
                fs::rename(entry.path(), target)?;
            }
        }

        Ok(())
    }

    /// Get list of files in package
    pub fn list_files(&self) -> crate::Result<Vec<String>> {
        let temp_dir = TempDir::new()?;
        self.extract(temp_dir.path())?;

        let manifest_path = temp_dir.path().join("metadata.json");
        let manifest_json = fs::read_to_string(manifest_path)?;
        let manifest: PackageManifest = serde_json::from_str(&manifest_json)
            .map_err(|e| crate::Error::Serialization(e.to_string()))?;

        Ok(manifest.files)
    }

    /// Verify package signature
    pub fn verify_signature(&self, public_key: &str) -> crate::Result<bool> {
        use crate::signature::SignatureVerifier;

        let verifier = SignatureVerifier::from_base64(public_key)?;

        // Read the package file
        let mut file = File::open(&self.path)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;

        // Verify signature
        verifier.verify(&contents, &self.metadata.signature).map(|_| true)
    }

    /// Get package manifest
    pub fn manifest(&self) -> crate::Result<PackageManifest> {
        let temp_dir = TempDir::new()?;
        self.extract(temp_dir.path())?;

        let manifest_path = temp_dir.path().join("metadata.json");
        let manifest_json = fs::read_to_string(manifest_path)?;
        serde_json::from_str(&manifest_json)
            .map_err(|e| crate::Error::Serialization(e.to_string()))
    }
}

/// Create a package from a directory
pub fn create_package(
    source_dir: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    manifest: PackageManifest,
) -> crate::Result<PackageArchive> {
    let source_dir = source_dir.as_ref();

    // Collect all files in the directory
    let mut files = Vec::new();
    if source_dir.exists() {
        for entry in walkdir::WalkDir::new(source_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                files.push(path.to_path_buf());
            }
        }
    }

    PackageArchive::create(output_path, manifest, &files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signature::KeyPair;

    #[test]
    fn test_manifest_creation() {
        let key = KeyPair::generate();
        let signature = key.sign(b"test");

        let manifest = PackageManifest::new(
            "test".to_string(),
            "1.0.0".to_string(),
            PackageKind::App,
            "x86_64".to_string(),
            1024,
            "0".repeat(64),
            "https://example.com/test.rpg".to_string(),
            signature,
        );

        assert_eq!(manifest.name, "test");
        assert_eq!(manifest.kind, "app");
    }
}
