// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! Cryptographic signing and verification for packages

use ed25519_dalek::{
    SecretKey, Signature as Ed25519Signature, SigningKey as Ed25519SigningKey, Signer,
    VerifyingKey, Verifier,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha512};
use base64::Engine as _;

/// A cryptographic signature for packages
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageSignature(pub [u8; 64]);

impl PackageSignature {
    /// Create a new signature from bytes
    pub fn new(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    /// Create a signature from a slice
    pub fn from_slice(slice: &[u8]) -> crate::Result<Self> {
        if slice.len() != 64 {
            return Err(crate::Error::SignatureVerification(
                "Invalid signature length".to_string(),
            ));
        }

        let mut bytes = [0u8; 64];
        bytes.copy_from_slice(slice);
        Ok(Self(bytes))
    }

    /// Get the signature as bytes
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }

    /// Encode the signature as base64
    pub fn to_base64(&self) -> String {
        base64::engine::general_purpose::STANDARD.encode(&self.0)
    }

    /// Decode a signature from base64
    pub fn from_base64(s: &str) -> crate::Result<Self> {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(s)
            .map_err(|_| {
                crate::Error::SignatureVerification("Invalid base64 encoding".to_string())
            })?;

        Self::from_slice(&bytes)
    }
}

// Implement Serialize for PackageSignature manually
impl Serialize for PackageSignature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_base64())
    }
}

// Implement Deserialize for PackageSignature manually
impl<'de> Deserialize<'de> for PackageSignature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_base64(&s).map_err(serde::de::Error::custom)
    }
}

/// Re-export as Signature for convenience
pub use PackageSignature as Signature;

/// A signing keypair
#[derive(Debug)]
pub struct KeyPair {
    /// The Ed25519 signing key
    signing_key: Ed25519SigningKey,
    /// The Ed25519 verifying key
    verifying_key: VerifyingKey,
}

impl KeyPair {
    /// Generate a new signing key
    pub fn generate() -> Self {
        let signing_key = Ed25519SigningKey::generate(&mut rand::rngs::OsRng);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Get the verifying key
    pub fn verifying_key(&self) -> &VerifyingKey {
        &self.verifying_key
    }

    /// Get the secret key bytes
    pub fn secret_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    /// Sign data
    pub fn sign(&self, data: &[u8]) -> PackageSignature {
        let signature = self.signing_key.sign(data);
        PackageSignature(signature.to_bytes())
    }

    /// Sign a hash
    pub fn sign_hash(&self, hash: &[u8; 64]) -> PackageSignature {
        let signature = self.signing_key.sign(hash);
        PackageSignature(signature.to_bytes())
    }

    /// Export the public key as base64
    pub fn export_public(&self) -> String {
        base64::engine::general_purpose::STANDARD.encode(self.verifying_key.as_bytes())
    }

    /// Export the secret key as base64 (WARNING: use with caution)
    pub fn export_secret(&self) -> String {
        base64::engine::general_purpose::STANDARD.encode(&self.signing_key.to_bytes())
    }

    /// Import a public key from base64
    pub fn import_public(s: &str) -> crate::Result<VerifyingKey> {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(s)
            .map_err(|_| {
                crate::Error::SignatureVerification("Invalid base64 encoding".to_string())
            })?;

        VerifyingKey::try_from(bytes.as_slice())
            .map_err(|_| {
                crate::Error::SignatureVerification("Invalid public key".to_string())
            })
    }

    /// Import a secret key from base64
    pub fn import_secret(s: &str) -> crate::Result<Self> {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(s)
            .map_err(|_| {
                crate::Error::SignatureVerification("Invalid base64 encoding".to_string())
            })?;

        if bytes.len() != 32 {
            return Err(crate::Error::SignatureVerification(
                "Invalid secret key length".to_string(),
            ));
        }

        // Convert Vec<u8> to [u8; 32]
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);

        // Derive public key from secret key
        let secret = SecretKey::from(array);
        let signing_key = Ed25519SigningKey::from(&secret);
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            signing_key,
            verifying_key,
        })
    }
}

/// Signature verifier
#[derive(Debug, Clone)]
pub struct SignatureVerifier {
    /// The public key
    public_key: VerifyingKey,
}

impl SignatureVerifier {
    /// Create a new verifier from a public key
    pub fn new(public_key: VerifyingKey) -> Self {
        Self { public_key }
    }

    /// Create a verifier from a base64-encoded public key
    pub fn from_base64(key: &str) -> crate::Result<Self> {
        Ok(Self {
            public_key: KeyPair::import_public(key)?,
        })
    }

    /// Verify a signature on data
    pub fn verify(&self, data: &[u8], signature: &PackageSignature) -> crate::Result<()> {
        let sig = Ed25519Signature::from_bytes(&signature.0);

        self.public_key
            .verify(data, &sig)
            .map_err(|_| {
                crate::Error::SignatureVerification("Invalid signature".to_string())
            })
    }

    /// Verify a signature on a hash
    pub fn verify_hash(&self, hash: &[u8; 64], signature: &PackageSignature) -> crate::Result<()> {
        let sig = Ed25519Signature::from_bytes(&signature.0);

        self.public_key
            .verify(hash, &sig)
            .map_err(|_| {
                crate::Error::SignatureVerification("Invalid signature".to_string())
            })
    }

    /// Compute SHA-512 hash of data
    pub fn hash(data: &[u8]) -> [u8; 64] {
        let mut hasher = Sha512::new();
        hasher.update(data);
        let result = hasher.finalize();
        let mut hash = [0u8; 64];
        hash.copy_from_slice(&result[..64]);
        hash
    }

    /// Verify a signature with hash
    pub fn verify_with_hash(
        &self,
        data: &[u8],
        signature: &PackageSignature,
    ) -> crate::Result<()> {
        let hash = Self::hash(data);
        self.verify_hash(&hash, signature)
    }

    /// Get the public key bytes
    pub fn public_key_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(self.public_key.as_bytes());
        bytes
    }

    /// Get the public key as base64
    pub fn public_key_base64(&self) -> String {
        base64::engine::general_purpose::STANDARD.encode(self.public_key.as_bytes())
    }
}

// Type alias for backward compatibility
pub type SigningKey = KeyPair;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let key = KeyPair::generate();
        let data = b"test data";

        let signature = key.sign(data);
        let verifier = SignatureVerifier::new(key.verifying_key().clone());

        assert!(verifier.verify(data, &signature).is_ok());
    }

    #[test]
    fn test_signature_encoding() {
        let key = KeyPair::generate();
        let data = b"test data";

        let signature = key.sign(data);
        let encoded = signature.to_base64();
        let decoded = PackageSignature::from_base64(&encoded).unwrap();

        assert_eq!(signature, decoded);
    }

    #[test]
    fn test_invalid_signature() {
        let key = KeyPair::generate();
        let data = b"test data";
        let wrong_data = b"wrong data";

        let signature = key.sign(data);
        let verifier = SignatureVerifier::new(key.verifying_key().clone());

        assert!(verifier.verify(wrong_data, &signature).is_err());
    }

    #[test]
    fn test_key_import_export() {
        let key = KeyPair::generate();
        let public_encoded = key.export_public();
        let public_imported = KeyPair::import_public(&public_encoded).unwrap();

        assert_eq!(key.verifying_key().as_bytes(), public_imported.as_bytes());

        let verifier = SignatureVerifier::new(key.verifying_key().clone());
        let verifier2 = SignatureVerifier::from_base64(&public_encoded).unwrap();

        let data = b"test";
        let signature = key.sign(data);

        assert!(verifier.verify(data, &signature).is_ok());
        assert!(verifier2.verify(data, &signature).is_ok());
    }
}
