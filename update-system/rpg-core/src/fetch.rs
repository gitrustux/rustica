// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! HTTP fetching for packages and repository indices

use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::time::timeout;

use crate::sources::Source;

/// Default timeout for HTTP requests (in seconds)
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Maximum number of retries for failed downloads
const MAX_RETRIES: usize = 3;

/// HTTP client configuration
#[derive(Debug, Clone)]
pub struct FetchOptions {
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Maximum number of retries
    pub max_retries: usize,
    /// Whether to verify SSL certificates (in production, always true)
    pub verify_ssl: bool,
    /// User agent string
    pub user_agent: String,
}

impl Default for FetchOptions {
    fn default() -> Self {
        Self {
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            max_retries: MAX_RETRIES,
            verify_ssl: true,
            user_agent: format!("RPG/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

/// Errors that can occur during fetching
#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    HttpError(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Timeout
    #[error("Request timed out after {0}s")]
    Timeout(u64),

    /// Invalid URL
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Checksum verification failed
    #[error("Checksum verification failed: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    /// All sources failed
    #[error("All sources failed to provide the resource")]
    AllSourcesFailed,

    /// File not found
    #[error("File not found: {0}")]
    NotFound(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<reqwest::Error> for FetchError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            FetchError::Timeout(DEFAULT_TIMEOUT_SECS)
        } else if err.is_connect() {
            FetchError::NetworkError(err.to_string())
        } else if err.is_request() {
            FetchError::HttpError(err.to_string())
        } else {
            FetchError::NetworkError(err.to_string())
        }
    }
}

/// Repository index from a source
#[derive(Debug, Clone, Deserialize)]
pub struct RepositoryIndex {
    /// Repository name
    pub name: String,
    /// Repository version
    pub version: String,
    /// Last update timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<i64>,
    /// Available packages
    pub packages: Vec<PackageEntry>,
}

/// Package entry in repository index
#[derive(Debug, Clone, Deserialize)]
pub struct PackageEntry {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Package description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Package size in bytes
    pub size: u64,
    /// SHA-256 checksum
    pub sha256: String,
    /// Package signature (base64)
    pub signature: String,
    /// Dependencies
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Relative path to package file
    pub path: String,
}

/// Download progress information
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// Total bytes to download
    pub total_bytes: u64,
    /// Bytes downloaded so far
    pub downloaded_bytes: u64,
    /// Download percentage (0-100)
    pub percentage: f64,
    /// Current speed in bytes/second
    pub bytes_per_second: f64,
}

/// Download result
#[derive(Debug)]
pub struct DownloadResult {
    /// Path where file was saved
    pub path: PathBuf,
    /// Total bytes downloaded
    pub total_bytes: u64,
    /// SHA-256 checksum of downloaded file
    pub checksum: String,
    /// Whether download was resumed from partial
    pub resumed: bool,
}

/// Fetch a repository index from multiple sources with failover
pub async fn fetch_index(
    sources: &[&Source],
    options: Option<FetchOptions>,
) -> Result<RepositoryIndex, FetchError> {
    let opts = options.unwrap_or_default();

    for source in sources {
        let url = source.index_url();
        match fetch_index_from_url(&url, &opts).await {
            Ok(index) => return Ok(index),
            Err(FetchError::NotFound(_)) => {
                // Try next source immediately for 404
                continue;
            }
            Err(FetchError::NetworkError(_) | FetchError::Timeout(_)) => {
                // Retry this source before moving to next
                let mut last_err = None;
                for retry in 1..=opts.max_retries {
                    match fetch_index_from_url(&url, &opts).await {
                        Ok(index) => return Ok(index),
                        Err(e) => {
                            last_err = Some(e);
                            tokio::time::sleep(Duration::from_secs(retry as u64)).await;
                        }
                    }
                }
                if let Some(err) = last_err {
                    log::warn!(
                        "Source {} failed after retries: {}",
                        source.name,
                        err
                    );
                }
            }
            Err(e) => {
                log::warn!("Source {} failed: {}", source.name, e);
            }
        }
    }

    Err(FetchError::AllSourcesFailed)
}

/// Fetch a repository index from a specific URL
async fn fetch_index_from_url(
    url: &str,
    options: &FetchOptions,
) -> Result<RepositoryIndex, FetchError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(options.timeout_secs))
        .user_agent(&options.user_agent)
        .build()
        .map_err(|e| FetchError::HttpError(e.to_string()))?;

    let response = timeout(
        Duration::from_secs(options.timeout_secs),
        client.get(url).send(),
    )
    .await
    .map_err(|_| FetchError::Timeout(options.timeout_secs))?
    .map_err(FetchError::from)?;

    if response.status() == 404 {
        return Err(FetchError::NotFound(url.to_string()));
    }

    if !response.status().is_success() {
        return Err(FetchError::HttpError(format!(
            "HTTP {}: {}",
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("Unknown")
        )));
    }

    let index = response
        .json()
        .await
        .map_err(|e| FetchError::HttpError(e.to_string()))?;

    Ok(index)
}

/// Fetch a package file from multiple sources with failover
pub async fn fetch_package(
    sources: &[&Source],
    package_name: &str,
    version: &str,
    expected_checksum: &str,
    output_path: &Path,
    options: Option<FetchOptions>,
    progress_callback: Option<Box<dyn Fn(DownloadProgress) + Send + Sync>>,
) -> Result<DownloadResult, FetchError> {
    let opts = options.unwrap_or_default();

    // Check if file already exists and is valid
    if output_path.exists() {
        if let Ok(existing_checksum) = compute_checksum(output_path) {
            if existing_checksum == expected_checksum {
                return Ok(DownloadResult {
                    path: output_path.to_path_buf(),
                    total_bytes: fs::metadata(output_path)?.len(),
                    checksum: existing_checksum,
                    resumed: true,
                });
            }
        }
        // File exists but is invalid, remove it
        fs::remove_file(output_path)?;
    }

    for source in sources {
        let url = source.package_url(package_name, version);
        match fetch_file_from_url(
            &url,
            output_path,
            expected_checksum,
            &opts,
            progress_callback.as_ref(),
        )
        .await
        {
            Ok(result) => return Ok(result),
            Err(FetchError::NotFound(_)) => {
                continue;
            }
            Err(FetchError::NetworkError(_) | FetchError::Timeout(_)) => {
                let mut last_err = None;
                for retry in 1..=opts.max_retries {
                    match fetch_file_from_url(
                        &url,
                        output_path,
                        expected_checksum,
                        &opts,
                        progress_callback.as_ref(),
                    )
                    .await
                    {
                        Ok(result) => return Ok(result),
                        Err(e) => {
                            last_err = Some(e);
                            tokio::time::sleep(Duration::from_secs(retry as u64)).await;
                        }
                    }
                }
                if let Some(err) = last_err {
                    log::warn!(
                        "Source {} failed for package {}: {}",
                        source.name,
                        package_name,
                        err
                    );
                }
            }
            Err(e) => {
                log::warn!(
                    "Source {} failed for package {}: {}",
                    source.name,
                    package_name,
                    e
                );
            }
        }
    }

    Err(FetchError::AllSourcesFailed)
}

/// Fetch a file from a specific URL
async fn fetch_file_from_url(
    url: &str,
    output_path: &Path,
    expected_checksum: &str,
    options: &FetchOptions,
    _progress_callback: Option<&Box<dyn Fn(DownloadProgress) + Send + Sync>>,
) -> Result<DownloadResult, FetchError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(options.timeout_secs))
        .user_agent(&options.user_agent)
        .build()
        .map_err(|e| FetchError::HttpError(e.to_string()))?;

    let response = timeout(
        Duration::from_secs(options.timeout_secs),
        client.get(url).send(),
    )
    .await
    .map_err(|_| FetchError::Timeout(options.timeout_secs))?
    .map_err(FetchError::from)?;

    if response.status() == 404 {
        return Err(FetchError::NotFound(url.to_string()));
    }

    if !response.status().is_success() {
        return Err(FetchError::HttpError(format!(
            "HTTP {}: {}",
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("Unknown")
        )));
    }

    let total_bytes = response
        .content_length()
        .ok_or_else(|| FetchError::HttpError("Missing Content-Length header".to_string()))?;

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Download file
    // Note: In production, would implement proper streaming with progress callback
    let bytes = timeout(
        Duration::from_secs(options.timeout_secs),
        reqwest::get(url),
    )
    .await
    .map_err(|_| FetchError::Timeout(options.timeout_secs))?
    .map_err(FetchError::from)?
    .bytes()
    .await
    .map_err(|e| FetchError::HttpError(e.to_string()))?;

    // Write to file
    let mut file = tokio::fs::File::create(output_path).await?;
    file.write_all(&bytes).await?;

    // Verify checksum
    let actual_checksum = checksum_bytes(&bytes);
    if actual_checksum != expected_checksum {
        fs::remove_file(output_path)?;
        return Err(FetchError::ChecksumMismatch {
            expected: expected_checksum.to_string(),
            actual: actual_checksum,
        });
    }

    Ok(DownloadResult {
        path: output_path.to_path_buf(),
        total_bytes,
        checksum: actual_checksum,
        resumed: false,
    })
}

/// Compute SHA-256 checksum of a file
pub fn compute_checksum(path: &Path) -> Result<String, FetchError> {
    let bytes = fs::read(path)?;
    Ok(checksum_bytes(&bytes))
}

/// Compute SHA-256 checksum of bytes
fn checksum_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    hex::encode(result)
}

/// Generic file fetch function (for backward compatibility)
pub async fn fetch_file(
    url: &str,
    output_path: &Path,
    options: Option<FetchOptions>,
) -> Result<PathBuf, FetchError> {
    let opts = options.unwrap_or_default();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(opts.timeout_secs))
        .user_agent(&opts.user_agent)
        .build()
        .map_err(|e| FetchError::HttpError(e.to_string()))?;

    let response = timeout(
        Duration::from_secs(opts.timeout_secs),
        client.get(url).send(),
    )
    .await
    .map_err(|_| FetchError::Timeout(opts.timeout_secs))?
    .map_err(FetchError::from)?;

    if response.status() == 404 {
        return Err(FetchError::NotFound(url.to_string()));
    }

    if !response.status().is_success() {
        return Err(FetchError::HttpError(format!(
            "HTTP {}: {}",
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("Unknown")
        )));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| FetchError::HttpError(e.to_string()))?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(output_path, bytes)?;
    Ok(output_path.to_path_buf())
}

/// Check if a URL is reachable
pub async fn check_url(url: &str, options: Option<FetchOptions>) -> bool {
    let opts = options.unwrap_or_default();

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(opts.timeout_secs))
        .user_agent(&opts.user_agent)
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    let result = timeout(
        Duration::from_secs(opts.timeout_secs),
        client.head(url).send(),
    )
    .await;

    match result {
        Ok(Ok(response)) => response.status().is_success() || response.status() == 405,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum() {
        let data = b"hello world";
        let checksum = checksum_bytes(data);
        assert_eq!(checksum, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
    }

    #[test]
    fn test_fetch_options_default() {
        let opts = FetchOptions::default();
        assert_eq!(opts.timeout_secs, DEFAULT_TIMEOUT_SECS);
        assert_eq!(opts.max_retries, MAX_RETRIES);
        assert!(opts.verify_ssl);
    }
}
