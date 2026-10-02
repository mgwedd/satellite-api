//! RS256 JWT Encoding & Decoding Module

use crate::auth::claims::Claims;
use crate::auth::keys::{get_rsa_private_key_pem, get_rsa_public_key_pem};
use crate::auth::mtls::CnfClaim;
use crate::error::AppError;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

/// Generates a strictly RS256-signed JWT token string and Claims struct with full OIDC/OAuth2 metadata.
pub fn create_jwt_token_full(
    sub: &str,
    role: &str,
    iss: Option<String>,
    aud: Option<String>,
    scope: Option<String>,
    cnf: Option<CnfClaim>,
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
        cnf,
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
        None,
        ttl_seconds,
    )
}

/// Generates a strictly RS256-signed JWT token bound to an RFC 8705 mTLS client certificate thumbprint.
pub fn create_jwt_token_bound(
    sub: &str,
    role: &str,
    cnf: Option<CnfClaim>,
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
        cnf,
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
