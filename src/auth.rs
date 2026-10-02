use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::AppError;

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

/// Modern OAuth 2.0 / OIDC Role Hierarchy for Satellite API RBAC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    /// Read-only access to satellite data and astrodynamics calculations.
    Viewer,
    /// Read and write access to create/update satellites and trigger sync pipelines.
    Editor,
    /// Full administrative access including destructive deletion.
    Admin,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Viewer => "viewer",
            UserRole::Editor => "editor",
            UserRole::Admin => "admin",
        }
    }
}

/// JWT Claims payload structure containing standard OIDC and OAuth 2.0 claim fields.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Claims {
    /// Subject (User ID, username, or client service ID)
    pub sub: String,
    /// Token issuer (OIDC issuer URI or auth server ID)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    /// Token audience (target API audience)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    /// Expiration timestamp in seconds since Unix epoch
    pub exp: usize,
    /// Issued-at timestamp in seconds since Unix epoch
    pub iat: usize,
    /// User role ("viewer", "editor", or "admin")
    pub role: String,
    /// Optional list of secondary roles
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
    /// Space-separated OAuth 2.0 scopes string (e.g. "read:satellites write:satellites admin:satellites")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

impl Claims {
    /// Evaluates role hierarchy level (Admin: 3 > Editor/Operator: 2 > Viewer/Reader: 1).
    pub fn role_level(&self) -> u8 {
        match self.role.to_lowercase().as_str() {
            "admin" | "superuser" => 3,
            "editor" | "operator" | "writer" => 2,
            "viewer" | "reader" | "user" => 1,
            _ => 0,
        }
    }

    /// Checks whether caller has at least the minimum required role in the hierarchy.
    pub fn has_role(&self, minimum_role: UserRole) -> bool {
        let req_level = match minimum_role {
            UserRole::Admin => 3,
            UserRole::Editor => 2,
            UserRole::Viewer => 1,
        };
        self.role_level() >= req_level
    }

    /// Enforces minimum role requirement. Returns `AppError::Forbidden` if caller has insufficient privileges.
    pub fn require_role(&self, minimum_role: UserRole) -> Result<(), AppError> {
        if self.has_role(minimum_role) {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!(
                "Action requires '{:?}' role or higher (caller has '{}')",
                minimum_role, self.role
            )))
        }
    }

    /// Checks if token contains a specific OAuth 2.0 scope.
    pub fn has_scope(&self, required_scope: &str) -> bool {
        if self.role_level() >= 3 {
            return true; // Admin bypasses scope check
        }
        if let Some(ref scope_str) = self.scope {
            scope_str.split_whitespace().any(|s| s == required_scope)
        } else {
            false
        }
    }
}

pub mod handlers;
pub mod provider;

pub use handlers::*;
pub use provider::*;

/// Request body for developer account registration.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SignupRequest {
    /// Developer email address
    pub email: String,
    /// Account password
    pub password: String,
    /// Requested role ("viewer", "editor", or "admin", defaults to "viewer")
    pub role: Option<String>,
}

/// Request body for developer account login.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    /// Developer email address or username
    pub email: String,
    /// Account password
    pub password: String,
}

/// Request body for RFC 7523 Private Key JWT Client Assertion token exchange (M2M authentication).
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClientAssertionRequest {
    /// OAuth 2.0 grant type (must be "client_credentials" or "urn:ietf:params:oauth:grant-type:jwt-bearer")
    pub grant_type: String,
    /// Client assertion type (must be "urn:ietf:params:oauth:client-assertion-type:jwt-bearer")
    pub client_assertion_type: String,
    /// RS256-signed JWT assertion token generated using client's RSA private key
    pub client_assertion: String,
    /// Optional requested OAuth 2.0 scope
    pub scope: Option<String>,
}

/// Response returned upon successful RS256 JWT token generation.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    /// Encoded RS256 JWT Bearer token
    pub token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Token lifetime in seconds (e.g. 86400)
    pub expires_in: usize,
    /// Decoded token claims
    pub claims: Claims,
}

/// Returns the RSA 2048 private key PEM for signing RS256 tokens.
///
/// ⚠️ **LOCAL DEVELOPMENT ONLY**:
/// Reading key material from file paths (`LOCAL_DEV_RSA_PRIVATE_KEY_FILE` / `.keys/rsa_private.pem`) or
/// falling back to in-memory ephemeral keys is strictly intended for local dev and testing.
///
/// **Production Security**:
/// Production services MUST NOT load private key files from disk. Instead, key material
/// should be managed securely via Secret Managers or Vaults (e.g. AWS Secrets Manager, HashiCorp Vault,
/// GCP Secret Manager, Railway/Supabase secrets) and injected directly into the `RSA_PRIVATE_KEY`
/// environment variable.
///
/// Resolution order:
/// 1. Direct PEM string from `RSA_PRIVATE_KEY` environment variable (Production / Vault injection).
/// 2. [LOCAL DEV ONLY] File path from `LOCAL_DEV_RSA_PRIVATE_KEY_FILE` (or fallback `RSA_PRIVATE_KEY_FILE`) environment variable or `.keys/rsa_private.pem`.
/// 3. [LOCAL DEV ONLY] Ephemeral in-memory RSA 2048 private key fallback.
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
/// should be injected directly via the `RSA_PUBLIC_KEY` environment variable or retrieved from JWKS / Vault.
///
/// Resolution order:
/// 1. Direct PEM string from `RSA_PUBLIC_KEY` environment variable (Production / Vault injection).
/// 2. [LOCAL DEV ONLY] File path from `LOCAL_DEV_RSA_PUBLIC_KEY_FILE` (or fallback `RSA_PUBLIC_KEY_FILE`) environment variable or `.keys/rsa_public.pem`.
/// 3. [LOCAL DEV ONLY] Ephemeral in-memory RSA 2048 public key fallback.
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

/// Generates a strictly RS256-signed JWT token string and Claims struct with full OIDC/OAuth2 metadata.
pub fn create_jwt_token_full(
    sub: &str,
    role: &str,
    iss: Option<String>,
    aud: Option<String>,
    scope: Option<String>,
    ttl_seconds: u64,
) -> Result<(String, Claims), AppError> {
    let sub_clean = sub.trim();
    if sub_clean.is_empty() {
        return Err(AppError::BadRequest("Token subject cannot be empty".into()));
    }

    let role_clean = role.trim();
    if role_clean.is_empty() {
        return Err(AppError::BadRequest("Token role cannot be empty".into()));
    }

    let now = chrono::Utc::now().timestamp() as usize;
    let exp = now + ttl_seconds as usize;
    let claims = Claims {
        sub: sub_clean.to_string(),
        iss,
        aud,
        exp,
        iat: now,
        role: role_clean.to_string(),
        roles: Some(vec![role_clean.to_string()]),
        scope,
    };

    let private_pem = get_rsa_private_key_pem();
    let encoding_key = EncodingKey::from_rsa_pem(private_pem.as_bytes())
        .map_err(|e| AppError::InternalServerError(format!("Invalid RSA Private Key: {}", e)))?;

    let header = Header::new(Algorithm::RS256);
    let token = encode(&header, &claims, &encoding_key).map_err(|e| {
        AppError::InternalServerError(format!("Failed to encode RS256 JWT token: {}", e))
    })?;

    Ok((token, claims))
}

/// Generates a strictly RS256-signed JWT token string and Claims struct with default scopes.
pub fn create_jwt_token(
    sub: &str,
    role: &str,
    ttl_seconds: u64,
) -> Result<(String, Claims), AppError> {
    let default_scope = match role.to_lowercase().as_str() {
        "admin" | "superuser" => "read:satellites write:satellites admin:satellites",
        "editor" | "operator" | "writer" => "read:satellites write:satellites",
        _ => "read:satellites",
    };
    create_jwt_token_full(
        sub,
        role,
        Some("astrea-sda-api".to_string()),
        Some("astrea-sda-api".to_string()),
        Some(default_scope.to_string()),
        ttl_seconds,
    )
}

/// Decodes and strictly verifies the RS256 signature and expiration of a JWT token string.
/// Strictly rejects non-RS256 algorithms (e.g. HS256, HS384, alg=none) to prevent algorithm confusion attacks.
pub fn decode_jwt_token(token: &str) -> Result<Claims, AppError> {
    let public_pem = get_rsa_public_key_pem();
    let decoding_key = DecodingKey::from_rsa_pem(public_pem.as_bytes())
        .map_err(|e| AppError::InternalServerError(format!("Invalid RSA Public Key: {}", e)))?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;
    validation.validate_aud = false;

    let token_data = decode::<Claims>(token, &decoding_key, &validation).map_err(|e| {
        AppError::Unauthorized(format!("Invalid or expired RS256 JWT token: {}", e))
    })?;

    if token_data.claims.sub.trim().is_empty() {
        return Err(AppError::Unauthorized(
            "JWT token contains empty subject".into(),
        ));
    }

    Ok(token_data.claims)
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".into()))?;

        let auth_header_trim = auth_header.trim();
        if auth_header_trim.len() < 7 || !auth_header_trim[..6].eq_ignore_ascii_case("bearer") {
            return Err(AppError::Unauthorized(
                "Invalid Authorization header scheme, expected 'Bearer <token>'".into(),
            ));
        }

        let token = auth_header_trim[6..].trim();
        if token.is_empty() {
            return Err(AppError::Unauthorized("Bearer token is empty".into()));
        }

        decode_jwt_token(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_create_and_decode_valid_rs256_token() {
        let (token, claims) =
            create_jwt_token("test_user", "admin", 3600).expect("Failed to create RS256 token");
        assert_eq!(claims.sub, "test_user");
        assert_eq!(claims.role, "admin");

        let decoded = decode_jwt_token(&token).expect("Failed to decode RS256 token");
        assert_eq!(decoded.sub, "test_user");
        assert_eq!(decoded.role, "admin");
    }

    #[test]
    fn test_jwt_rejects_hs256_token_algorithm_confusion() {
        let now = chrono::Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: "attacker".into(),
            iss: None,
            aud: None,
            exp: now + 3600,
            iat: now,
            role: "admin".into(),
            roles: Some(vec!["admin".into()]),
            scope: Some("read:satellites write:satellites admin:satellites".into()),
        };

        // Attempt to encode using HMAC-SHA256 (HS256) instead of RS256
        let hs256_secret = b"some_hmac_secret_key_used_by_attacker";
        let hs256_token = encode(
            &Header::default(), // Defaults to HS256
            &claims,
            &EncodingKey::from_secret(hs256_secret),
        )
        .unwrap();

        // Attempt to decode with server's decode_jwt_token (strictly expects RS256)
        let res = decode_jwt_token(&hs256_token);
        assert!(
            res.is_err(),
            "Server MUST reject HS256 token when RS256 is strictly required"
        );
    }

    #[test]
    fn test_jwt_decode_expired_token() {
        let now = chrono::Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: "test_user".into(),
            iss: None,
            aud: None,
            exp: now - 100, // Expired 100s ago
            iat: now - 200,
            role: "admin".into(),
            roles: Some(vec!["admin".into()]),
            scope: Some("read:satellites write:satellites admin:satellites".into()),
        };

        let private_pem = get_rsa_private_key_pem();
        let encoding_key = EncodingKey::from_rsa_pem(private_pem.as_bytes()).unwrap();
        let token = encode(&Header::new(Algorithm::RS256), &claims, &encoding_key).unwrap();

        let res = decode_jwt_token(&token);
        assert!(res.is_err());
    }

    #[test]
    fn test_rbac_role_requirements() {
        let admin_claims = Claims {
            sub: "admin_user".into(),
            iss: None,
            aud: None,
            exp: 9999999999,
            iat: 1000000000,
            role: "admin".into(),
            roles: Some(vec!["admin".into()]),
            scope: Some("read:satellites write:satellites admin:satellites".into()),
        };
        assert!(admin_claims.require_role(UserRole::Admin).is_ok());
        assert!(admin_claims.require_role(UserRole::Editor).is_ok());

        let operator_claims = Claims {
            sub: "op_user".into(),
            iss: None,
            aud: None,
            exp: 9999999999,
            iat: 1000000000,
            role: "operator".into(),
            roles: Some(vec!["operator".into()]),
            scope: Some("read:satellites write:satellites".into()),
        };
        assert!(operator_claims.require_role(UserRole::Admin).is_err());
        assert!(operator_claims.require_role(UserRole::Editor).is_ok());

        let viewer_claims = Claims {
            sub: "viewer_user".into(),
            iss: None,
            aud: None,
            exp: 9999999999,
            iat: 1000000000,
            role: "viewer".into(),
            roles: Some(vec!["viewer".into()]),
            scope: Some("read:satellites".into()),
        };
        assert!(viewer_claims.require_role(UserRole::Viewer).is_ok());
        assert!(viewer_claims.require_role(UserRole::Editor).is_err());
        assert!(viewer_claims.require_role(UserRole::Admin).is_err());
    }
}
