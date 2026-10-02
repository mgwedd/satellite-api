//! RSA Key Management Module
//!
//! Separates production secret injection (Vault / Secret Manager environment variables)
//! from local development file path fallbacks (`LOCAL_DEV_RSA_***`).

use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use rsa::{RsaPrivateKey, RsaPublicKey};
use std::sync::OnceLock;

/// Ephemeral in-memory RSA 2048 key pair generated lazily for development and testing environments
/// when no environment variables or key files are provided.
struct EphemeralKeyPair {
    private_pem: String,
    public_pem: String,
}

static EPHEMERAL_KEY_PAIR: OnceLock<EphemeralKeyPair> = OnceLock::new();

fn get_ephemeral_key_pair() -> &'static EphemeralKeyPair {
    EPHEMERAL_KEY_PAIR.get_or_init(|| {
        let mut rng = rand::thread_rng();
        let bits = 2048;
        let private_key = RsaPrivateKey::new(&mut rng, bits)
            .expect("Failed to generate ephemeral RSA 2048 private key");
        let public_key = RsaPublicKey::from(&private_key);

        let private_pem = private_key
            .to_pkcs8_pem(LineEnding::LF)
            .expect("Failed to encode ephemeral private key to PKCS8 PEM")
            .to_string();

        let public_pem = public_key
            .to_public_key_pem(LineEnding::LF)
            .expect("Failed to encode ephemeral public key to PEM")
            .to_string();

        EphemeralKeyPair {
            private_pem,
            public_pem,
        }
    })
}

/// Returns the RSA 2048 private key PEM for signing RS256 tokens.
///
/// ⚠️ **LOCAL DEVELOPMENT ONLY**:
/// Reading key material from file paths (`LOCAL_DEV_RSA_PRIVATE_KEY_FILE` / `.keys/rsa_private.pem`) or
/// falling back to in-memory ephemeral keys is strictly intended for local dev and testing.
///
/// **Production Security**:
/// Production services MUST NOT load private key files from disk. Instead, key material
/// should be managed securely via Secret Managers or Vaults (AWS Secrets Manager, HashiCorp Vault,
/// GCP Secret Manager) and injected directly into `RSA_PRIVATE_KEY`.
pub fn get_rsa_private_key_pem() -> String {
    if let Ok(pem) = std::env::var("RSA_PRIVATE_KEY") {
        if !pem.trim().is_empty() {
            return pem;
        }
    }

    let file_path = std::env::var("LOCAL_DEV_RSA_PRIVATE_KEY_FILE")
        .or_else(|_| std::env::var("RSA_PRIVATE_KEY_FILE"))
        .unwrap_or_else(|_| ".keys/rsa_private.pem".to_string());
    if let Ok(contents) = std::fs::read_to_string(&file_path) {
        if !contents.trim().is_empty() {
            tracing::debug!(
                "Loaded RSA private key from local dev file path: {}",
                file_path
            );
            return contents;
        }
    }

    tracing::debug!("Using ephemeral in-memory RSA private key fallback (Local Dev Only)");
    get_ephemeral_key_pair().private_pem.clone()
}

/// Returns the RSA 2048 public key PEM for verifying RS256 signatures.
///
/// ⚠️ **LOCAL DEVELOPMENT ONLY**:
/// Reading key material from file paths (`LOCAL_DEV_RSA_PUBLIC_KEY_FILE` / `.keys/rsa_public.pem`) or
/// falling back to in-memory ephemeral keys is strictly intended for local dev and testing.
///
/// **Production Security**:
/// Production services MUST NOT load public key files from disk. Instead, public key material
/// should be injected directly via `RSA_PUBLIC_KEY` or retrieved from JWKS / Vault.
pub fn get_rsa_public_key_pem() -> String {
    if let Ok(pem) = std::env::var("RSA_PUBLIC_KEY") {
        if !pem.trim().is_empty() {
            return pem;
        }
    }

    let file_path = std::env::var("LOCAL_DEV_RSA_PUBLIC_KEY_FILE")
        .or_else(|_| std::env::var("RSA_PUBLIC_KEY_FILE"))
        .unwrap_or_else(|_| ".keys/rsa_public.pem".to_string());
    if let Ok(contents) = std::fs::read_to_string(&file_path) {
        if !contents.trim().is_empty() {
            tracing::debug!(
                "Loaded RSA public key from local dev file path: {}",
                file_path
            );
            return contents;
        }
    }

    tracing::debug!("Using ephemeral in-memory RSA public key fallback (Local Dev Only)");
    get_ephemeral_key_pair().public_pem.clone()
}
