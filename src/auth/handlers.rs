use crate::auth::provider::AuthProvider;
use crate::auth::{AuthResponse, Claims, LoginRequest, SignupRequest};
use crate::error::AppError;
use axum::{extract::State, Json};
use std::sync::Arc;

/// Developer Account Registration
///
/// Registers a new developer account and issues an authenticated RS256/Supabase JWT Bearer token.
#[utoipa::path(
    post,
    path = "/v1/auth/signup",
    operation_id = "signupHandler",
    request_body = SignupRequest,
    responses(
        (status = 200, description = "Developer account registered and JWT token issued", body = AuthResponse),
        (status = 400, description = "Invalid registration request or email already exists", body = ErrorResponse)
    ),
    tag = "Authentication"
)]
pub async fn signup_handler(
    State(provider): State<Arc<dyn AuthProvider>>,
    Json(payload): Json<SignupRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let res = provider.signup(&payload).await?;
    Ok(Json(res))
}

/// Developer Account Login & JWT Token Generation
///
/// Authenticates developer account credentials and returns an RS256/Supabase JWT Bearer token for API access and Swagger UI authorization.
#[utoipa::path(
    post,
    path = "/v1/auth/login",
    operation_id = "loginHandler",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Authenticated successfully and JWT token issued", body = AuthResponse),
        (status = 401, description = "Unauthorized - Invalid email or password", body = ErrorResponse)
    ),
    tag = "Authentication"
)]
pub async fn login_handler(
    State(provider): State<Arc<dyn AuthProvider>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let res = provider.login(&payload).await?;
    Ok(Json(res))
}

/// Get Current Authenticated Developer Profile
///
/// Returns decoded JWT claims and profile info for the authenticated caller.
#[utoipa::path(
    get,
    path = "/v1/auth/me",
    operation_id = "getCurrentUser",
    responses(
        (status = 200, description = "Decoded caller JWT claims", body = Claims),
        (status = 401, description = "Unauthorized - Invalid or missing Bearer token", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "Authentication"
)]
pub async fn me_handler(claims: Claims) -> Result<Json<Claims>, AppError> {
    Ok(Json(claims))
}
