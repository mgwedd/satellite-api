//! Authentication Request & Response Data Transfer Objects (DTOs)

use crate::auth::claims::Claims;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
