// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! Version management for packages and systems

use semver::{Version as SemverVersion, VersionReq};
use serde::{Deserialize, Serialize};

/// A semantic version with optional build metadata
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Version {
    /// The semantic version
    pub semver: SemverVersion,
    /// Optional pre-release identifier (e.g., "beta", "rc1")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre: Option<String>,
    /// Optional build metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<String>,
}

impl Version {
    /// Create a new version from semver
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            semver: SemverVersion::new(major, minor, patch),
            pre: None,
            build: None,
        }
    }

    /// Create a new version with pre-release identifier
    pub fn with_pre(major: u64, minor: u64, patch: u64, pre: &str) -> Self {
        Self {
            semver: SemverVersion::new(major, minor, patch),
            pre: Some(pre.to_string()),
            build: None,
        }
    }

    /// Parse a version from a string
    pub fn parse(s: &str) -> crate::Result<Self> {
        // Try to parse as semver
        let semver = SemverVersion::parse(s)
            .map_err(|_| crate::Error::InvalidVersion(s.to_string()))?;

        Ok(Self {
            semver,
            pre: None,
            build: None,
        })
    }

    /// Get the version as a string
    pub fn as_str(&self) -> String {
        self.semver.to_string()
    }

    /// Check if this is a pre-release version
    pub fn is_prerelease(&self) -> bool {
        self.semver.pre.is_empty()
    }

    /// Get the next major version
    pub fn next_major(&self) -> Self {
        Self {
            semver: SemverVersion::new(self.semver.major + 1, 0, 0),
            pre: None,
            build: None,
        }
    }

    /// Get the next minor version
    pub fn next_minor(&self) -> Self {
        Self {
            semver: SemverVersion::new(self.semver.major, self.semver.minor + 1, 0),
            pre: None,
            build: None,
        }
    }

    /// Get the next patch version
    pub fn next_patch(&self) -> Self {
        Self {
            semver: SemverVersion::new(
                self.semver.major,
                self.semver.minor,
                self.semver.patch + 1,
            ),
            pre: None,
            build: None,
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.semver)
    }
}

impl From<SemverVersion> for Version {
    fn from(semver: SemverVersion) -> Self {
        Self {
            semver,
            pre: None,
            build: None,
        }
    }
}

/// Version constraint for dependency resolution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionConstraint {
    /// The version requirement (e.g., "^1.0.0", "~2.1.0")
    pub requirement: String,
}

impl VersionConstraint {
    /// Create a new version constraint
    pub fn new(requirement: &str) -> crate::Result<Self> {
        // Validate the requirement
        VersionReq::parse(requirement)
            .map_err(|_| crate::Error::InvalidVersion(requirement.to_string()))?;

        Ok(Self {
            requirement: requirement.to_string(),
        })
    }

    /// Check if a version satisfies this constraint
    pub fn satisfies(&self, version: &Version) -> bool {
        let req = VersionReq::parse(&self.requirement).unwrap(); // We validated in new()
        req.matches(&version.semver)
    }

    /// Exact version constraint
    pub fn exact(version: &Version) -> Self {
        Self {
            requirement: format!("={}", version.semver),
        }
    }

    /// Caret version constraint (^)
    pub fn caret(version: &Version) -> Self {
        Self {
            requirement: format!("^{}", version.semver),
        }
    }

    /// Tilde version constraint (~)
    pub fn tilde(version: &Version) -> Self {
        Self {
            requirement: format!("~{}", version.semver),
        }
    }

    /// Greater than constraint
    pub fn greater_than(version: &Version) -> Self {
        Self {
            requirement: format!(">{}", version.semver),
        }
    }

    /// Greater than or equal constraint
    pub fn greater_or_equal(version: &Version) -> Self {
        Self {
            requirement: format!(">={}", version.semver),
        }
    }

    /// Less than constraint
    pub fn less_than(version: &Version) -> Self {
        Self {
            requirement: format!("<{}", version.semver),
        }
    }

    /// Less than or equal constraint
    pub fn less_or_equal(version: &Version) -> Self {
        Self {
            requirement: format!("<={}", version.semver),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parse() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.semver.major, 1);
        assert_eq!(v.semver.minor, 2);
        assert_eq!(v.semver.patch, 3);
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::new(1, 2, 3);
        let v2 = Version::new(1, 2, 4);
        assert!(v1 < v2);
    }

    #[test]
    fn test_version_next() {
        let v = Version::new(1, 2, 3);
        assert_eq!(v.next_patch(), Version::new(1, 2, 4));
        assert_eq!(v.next_minor(), Version::new(1, 3, 0));
        assert_eq!(v.next_major(), Version::new(2, 0, 0));
    }

    #[test]
    fn test_constraint_satisfies() {
        let v = Version::new(1, 2, 3);
        let caret = VersionConstraint::caret(&Version::new(1, 2, 0));
        assert!(caret.satisfies(&v));

        let exact = VersionConstraint::exact(&Version::new(1, 2, 3));
        assert!(exact.satisfies(&v));

        let wrong = VersionConstraint::exact(&Version::new(1, 2, 4));
        assert!(!wrong.satisfies(&v));
    }
}
