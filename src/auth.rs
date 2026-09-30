use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
    Json,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::AppError;

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

/// Request body for JWT authentication login/token generation endpoint.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    /// Username or service subject identifier
    pub username: String,
    /// Optional user role (defaults to "operator" if omitted)
    pub role: Option<String>,
}

/// Response returned upon successful JWT token generation.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    /// Encoded JWT Bearer token
    pub token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Token lifetime in seconds (e.g. 86400)
    pub expires_in: usize,
    /// Decoded token claims
    pub claims: Claims,
}

/// Returns the configured JWT secret key from `JWT_SECRET` environment variable,
/// falling back to a default development secret if unconfigured.
pub fn get_jwt_secret() -> String {
    std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "satellite_api_default_jwt_secret_key_change_in_prod".to_string())
}

/// Generates a signed JWT token string and Claims struct for a given subject and role.
pub fn create_jwt_token(
    sub: &str,
    role: &str,
    ttl_seconds: u64,
) -> Result<(String, Claims), AppError> {
    let now = chrono::Utc::now().timestamp() as usize;
    let exp = now + ttl_seconds as usize;
    let claims = Claims {
        sub: sub.to_string(),
        exp,
        iat: now,
        role: role.to_string(),
    };

    let secret = get_jwt_secret();
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::InternalServerError(format!("Failed to encode JWT token: {}", e)))?;

    Ok((token, claims))
}

/// Decodes and validates signature and expiration of a JWT token string.
pub fn decode_jwt_token(token: &str) -> Result<Claims, AppError> {
    let secret = get_jwt_secret();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Invalid or expired JWT token".into()))?;

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

        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized("Invalid token format".into()));
        }

        let token = auth_header["Bearer ".len()..].trim();
        decode_jwt_token(token)
    }
}

/// OpenAPI endpoint handler to issue a JWT authentication token for testing or client auth.
#[utoipa::path(
    post,
    path = "/v1/auth/login",
    operation_id = "loginHandler",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "JWT Token issued successfully", body = AuthResponse),
        (status = 400, description = "Invalid request payload", body = ErrorResponse)
    ),
    tag = "Authentication"
)]

pub async fn login_handler(
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    if payload.username.trim().is_empty() {
        return Err(AppError::BadRequest("Username cannot be empty".into()));
    }

    let role = payload.role.unwrap_or_else(|| "operator".to_string());
    let ttl_seconds = 86400; // 24 hours
    let (token, claims) = create_jwt_token(&payload.username, &role, ttl_seconds)?;

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
    fn test_jwt_create_and_decode_valid_token() {
        let (token, claims) =
            create_jwt_token("test_user", "admin", 3600).expect("Failed to create token");
        assert_eq!(claims.sub, "test_user");
        assert_eq!(claims.role, "admin");

        let decoded = decode_jwt_token(&token).expect("Failed to decode token");
        assert_eq!(decoded.sub, "test_user");
        assert_eq!(decoded.role, "admin");
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

        let secret = get_jwt_secret();
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        let res = decode_jwt_token(&token);
        assert!(res.is_err());
        match res {
            Err(AppError::Unauthorized(msg)) => assert!(msg.contains("Invalid or expired")),
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[test]
    fn test_jwt_decode_invalid_signature() {
        let (token, _) = create_jwt_token("test_user", "admin", 3600).unwrap();
        // Tamper with secret by using a different key
        let tampered_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(b"wrong_secret_key"),
            &Validation::default(),
        );
        assert!(tampered_data.is_err());
    }
}
