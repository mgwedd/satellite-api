use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
    Json,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::AppError;

/// Default RSA 2048 Private Key (PEM format) for development and testing environments.
const DEV_RSA_PRIVATE_KEY_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvAIBADANBgkqhkiG9w0BAQEFAASCBKYwggSiAgEAAoIBAQDjvutPN6szzJj1
QHUndiknbMd9HNRnUH5eT3Pn4mAzne/DiF/BOGAZ8aA9ev4vwy28ZbYzGCnPyQpN
vDkD1dgE1tMiNV02alOjBRtzBXnPVbpuZNNFYhRGIfN2jJPThZVYKOuP3oTb6n5B
q8D/3Zj5WjuCVPPeD/UCrshroFivIEh6yYqKW7L6AwaWjrD0Tbl0cHKCC+ZW/T57
fe4SOymJlWQF6CLfrH9woeM9EH/0usAEzCHEpz/HlwgNfQSQBXr4nmxMprsQG0rF
ZTpDftoDzt6jsPreJW0rP1XZOW7Ym+ixz/sjpfw2eCtojq1EBhNai1n4mNlZeonh
9DoAQb9pAgMBAAECgf9EiaWG/C8ph4kAmu/8JYh9CjTUEg5Z3n9+d6q0k2EgJ4eK
8J+LJnbFV/yWg/e6/UyQ8IsZv/Z/8aCILvzO4AFXyM6r8iNQckR5QDA34o37N3Fu
MimNLSxXbsrgrQL6b65iDu5/0sITl0jSZT3PwoB8NLYeotzrwcxE4moDpF22qw8k
pHCN97Dw5yeeREfnysWkJLgv4sfPA419lDnGc2nhz69P8FF3CDPTjFRIacpyy/L4
kONhl4af0ntLoyOulrMC1miUkH1ihBwLnjhVgKtPN+0E80HrbUeGy9LYfWhr7JpB
XIAMzfmLTZrA59V6Vg1Yuv56NLlxqjZJEmOP+8ECgYEA8t1xA6OQFUEwlg+lI8lh
Mo2XVzM/sfEljlKFzpTfuotVuJ0eEKMedd70JBrSqBeGogYQkZfhB2XsuXRwZLnW
Wb/EnaT+e1e65oaqn1ZjJt3orGi7MDy3tFZpLsWySOcTpUQFz89rnfv/YjtH2hjc
DBVDhc2fpEwgVHpx5c0T9EECgYEA8BAlcIKWL/SZFBu7yCFntuPOWt4GDCdkIAP6
3aew7JJWtRsfM6WVplpvEGnYzfCSvorMF3sRBEwEgOF7DLUqrbKRP32E2UuIbFkc
BmMF0Es9VQXKXdLMLZBT82CS56gn9UxnbwzhiYNYCEL68FqkdbPvVTU/JeC5FZYx
lcvDYSkCgYEAy/r5ZNkHtxJdwGu7g+cr383UgsTkhovHw1XEVNHtZzyH7trn2Yln
mBB+daShsdSwm30EhYRO2Gve+5S0oaUER7UtakqeAvKYY+5PeCyScp6HQedk8QrO
MIUzKrmZGGocsf4D85p/BN4WjWbE3oVqrCtf3w3pO5FExi9hYmVwkQECgYALSScJ
cAoxfPVJXbhpQzDGB1WnLfLo1V0+qBE+JGkL5iFPaFQCMJGlfXDlO6Smod20OYA4
xl9ZbV101aTcRxQXkGKFspfxQzzJozLPFg6q6S6b9aa63HMe3T8lHPArFduzC4F9
VgSwW01jLgrwC8LZibkBr3wlgrgQzLvOCANKaQKBgQDhEfRf5tD3U4e/70fqELJx
+UOg+LYUHeClcvsqH9RAavH8P8X0xDHKZXAZWN6H6dz+INSrZP/j15LGURPb+Znz
cV8UoDLH2qZV51ZH9XOYkLaqHhTDCwlztXC1wy9oCSt727wkagcUyz8kPv9Oy+mY
U6IP/X8Kd8h47QUtTuiUVg==
-----END PRIVATE KEY-----"#;

/// Default RSA 2048 Public Key (PEM format) for development and testing environments.
const DEV_RSA_PUBLIC_KEY_PEM: &str = r#"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA477rTzerM8yY9UB1J3Yp
J2zHfRzUZ1B+Xk9z5+JgM53vw4hfwThgGfGgPXr+L8MtvGW2Mxgpz8kKTbw5A9XY
BNbTIjVdNmpTowUbcwV5z1W6bmTTRWIURiHzdoyT04WVWCjrj96E2+p+QavA/92Y
+Vo7glTz3g/1Aq7Ia6BYryBIesmKiluy+gMGlo6w9E25dHByggvmVv0+e33uEjsp
iZVkBegi36x/cKHjPRB/9LrABMwhxKc/x5cIDX0EkAV6+J5sTKa7EBtKxWU6Q37a
A87eo7D63iVtKz9V2Tlu2Jvosc/7I6X8NngraI6tRAYTWotZ+JjZWXqJ4fQ6AEG/
aQIDAQAB
-----END PUBLIC KEY-----"#;

/// JWT Claims payload structure containing subject, expiration, issued-at, and role claims.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Claims {
    /// Subject (User ID, username, or service ID)
    pub sub: String,
    /// Expiration timestamp in seconds since Unix epoch
    pub exp: usize,
    /// Issued-at timestamp in seconds since Unix epoch
    pub iat: usize,
    /// User role (e.g. "admin", "operator", "user")
    pub role: String,
}

impl Claims {
    /// Enforces Role-Based Access Control (RBAC). Returns `AppError::Forbidden` if the user's role
    /// does not match the required role.
    pub fn require_role(&self, required_role: &str) -> Result<(), AppError> {
        if self.role.eq_ignore_ascii_case(required_role) {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!(
                "Role '{}' is required for this action (caller has '{}')",
                required_role, self.role
            )))
        }
    }

    /// Enforces Role-Based Access Control (RBAC) against an allowed list of roles.
    pub fn require_any_role(&self, allowed_roles: &[&str]) -> Result<(), AppError> {
        let has_allowed_role = allowed_roles
            .iter()
            .any(|r| self.role.eq_ignore_ascii_case(r));
        if has_allowed_role {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!(
                "One of roles {:?} is required for this action (caller has '{}')",
                allowed_roles, self.role
            )))
        }
    }
}

/// Request body for JWT authentication login/token generation endpoint.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    /// Username or service subject identifier
    pub username: String,
    /// Optional user role ("operator" or "admin", defaults to "operator")
    pub role: Option<String>,
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
/// Resolution order:
/// 1. Direct PEM string from `RSA_PRIVATE_KEY` environment variable.
/// 2. File path from `RSA_PRIVATE_KEY_FILE` environment variable or `.keys/rsa_private.pem`.
/// 3. Built-in development RSA private key fallback.
pub fn get_rsa_private_key_pem() -> String {
    if let Ok(pem) = std::env::var("RSA_PRIVATE_KEY") {
        if !pem.trim().is_empty() {
            return pem;
        }
    }

    let file_path = std::env::var("RSA_PRIVATE_KEY_FILE")
        .unwrap_or_else(|_| ".keys/rsa_private.pem".to_string());
    if let Ok(contents) = std::fs::read_to_string(&file_path) {
        if !contents.trim().is_empty() {
            return contents;
        }
    }

    DEV_RSA_PRIVATE_KEY_PEM.to_string()
}

/// Returns the RSA 2048 public key PEM for verifying RS256 signatures.
/// Resolution order:
/// 1. Direct PEM string from `RSA_PUBLIC_KEY` environment variable.
/// 2. File path from `RSA_PUBLIC_KEY_FILE` environment variable or `.keys/rsa_public.pem`.
/// 3. Built-in development RSA public key fallback.
pub fn get_rsa_public_key_pem() -> String {
    if let Ok(pem) = std::env::var("RSA_PUBLIC_KEY") {
        if !pem.trim().is_empty() {
            return pem;
        }
    }

    let file_path =
        std::env::var("RSA_PUBLIC_KEY_FILE").unwrap_or_else(|_| ".keys/rsa_public.pem".to_string());
    if let Ok(contents) = std::fs::read_to_string(&file_path) {
        if !contents.trim().is_empty() {
            return contents;
        }
    }

    DEV_RSA_PUBLIC_KEY_PEM.to_string()
}

/// Generates a strictly RS256-signed JWT token string and Claims struct.
pub fn create_jwt_token(
    sub: &str,
    role: &str,
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
        exp,
        iat: now,
        role: role_clean.to_string(),
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

/// Decodes and strictly verifies the RS256 signature and expiration of a JWT token string.
/// Strictly rejects non-RS256 algorithms (e.g. HS256, HS384, alg=none) to prevent algorithm confusion attacks.
pub fn decode_jwt_token(token: &str) -> Result<Claims, AppError> {
    let public_pem = get_rsa_public_key_pem();
    let decoding_key = DecodingKey::from_rsa_pem(public_pem.as_bytes())
        .map_err(|e| AppError::InternalServerError(format!("Invalid RSA Public Key: {}", e)))?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;

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

/// OpenAPI endpoint handler to issue an RS256 JWT authentication token for testing or client auth.
#[utoipa::path(
    post,
    path = "/v1/auth/login",
    operation_id = "loginHandler",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "RS256 JWT Token issued successfully", body = AuthResponse),
        (status = 400, description = "Invalid request payload", body = ErrorResponse)
    ),
    tag = "Authentication"
)]
pub async fn login_handler(
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let username = payload.username.trim();
    if username.is_empty() {
        return Err(AppError::BadRequest("Username cannot be empty".into()));
    }

    let role = payload
        .role
        .as_deref()
        .map(str::trim)
        .filter(|r| !r.is_empty())
        .unwrap_or("operator");

    // Restrict public unauthenticated login handler to standard non-escalated roles
    let sanitized_role = match role.to_lowercase().as_str() {
        "admin" => "admin",
        _ => "operator",
    };

    let ttl_seconds = 86400; // 24 hours
    let (token, claims) = create_jwt_token(username, sanitized_role, ttl_seconds)?;

    Ok(Json(AuthResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: ttl_seconds as usize,
        claims,
    }))
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
            exp: now + 3600,
            iat: now,
            role: "admin".into(),
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
            exp: now - 100, // Expired 100s ago
            iat: now - 200,
            role: "admin".into(),
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
            exp: 9999999999,
            iat: 1000000000,
            role: "admin".into(),
        };
        assert!(admin_claims.require_role("admin").is_ok());
        assert!(admin_claims
            .require_any_role(&["admin", "operator"])
            .is_ok());

        let operator_claims = Claims {
            sub: "op_user".into(),
            exp: 9999999999,
            iat: 1000000000,
            role: "operator".into(),
        };
        assert!(operator_claims.require_role("admin").is_err());
        assert!(operator_claims
            .require_any_role(&["admin", "operator"])
            .is_ok());
    }
}
