//! RFC 8705 Mutual TLS (mTLS) Certificate Binding Module
//!
//! Provides OAuth 2.0 / OIDC Certificate-Bound Access Tokens (`cnf` claim)
//! and verifies client X.509 certificate fingerprints.

use crate::error::AppError;
use axum::http::request::Parts;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// RFC 8705 OIDC Confirmation Claim (`cnf`) containing SHA-256 certificate thumbprint (`x5t#S256`).
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct CnfClaim {
    /// SHA-256 base64url-encoded client certificate thumbprint
    #[serde(rename = "x5t#S256")]
    pub x5t_s256: String,
}

impl CnfClaim {
    pub fn new(x5t_s256: String) -> Self {
        Self { x5t_s256 }
    }
}

/// Extracts client certificate fingerprint from request headers (e.g. `X-Client-Cert-Fingerprint` or `X-Client-Cert-Hash`).
pub fn extract_client_cert_fingerprint(parts: &Parts) -> Option<String> {
    parts
        .headers
        .get("x-client-cert-fingerprint")
        .or_else(|| parts.headers.get("x-client-cert-hash"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
}

/// Verifies RFC 8705 Certificate Binding for incoming HTTP requests.
/// If token contains a `cnf` claim, mandates that caller's TLS client certificate fingerprint matches.
pub fn verify_mtls_cert_binding(parts: &Parts, cnf: &CnfClaim) -> Result<(), AppError> {
    let client_fingerprint = extract_client_cert_fingerprint(parts);

    match client_fingerprint {
        Some(fingerprint) => {
            if fingerprint != cnf.x5t_s256 {
                Err(AppError::Unauthorized(
                    "mTLS Certificate Binding Mismatch: Bearer token is bound to a different client X.509 certificate".into(),
                ))
            } else {
                Ok(())
            }
        }
        None => Err(AppError::Unauthorized(
            "mTLS Certificate Missing: Token is RFC 8705 certificate-bound but request lacked X-Client-Cert-Fingerprint".into(),
        )),
    }
}
