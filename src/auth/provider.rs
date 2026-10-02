use crate::auth::claims::Claims;
use crate::auth::jwt::{create_jwt_token_full, decode_jwt_token};
use crate::auth::mtls::CnfClaim;
use crate::auth::{AuthResponse, ClientAssertionRequest, LoginRequest, SignupRequest};
use crate::error::AppError;
use axum::async_trait;
use serde::Deserialize;
use sqlx::Row;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn signup(&self, req: &SignupRequest) -> Result<AuthResponse, AppError>;
    async fn login(&self, req: &LoginRequest) -> Result<AuthResponse, AppError>;
    async fn client_assertion_token_exchange(
        &self,
        req: &ClientAssertionRequest,
        client_cert_fingerprint: Option<String>,
    ) -> Result<AuthResponse, AppError>;
    async fn verify_token(&self, token: &str) -> Result<Claims, AppError>;
}

/// In-Memory AuthProvider for local development, pre-seeded dev accounts, and offline testing.
pub struct MemoryAuthProvider {
    users: Arc<RwLock<HashMap<String, (String, String)>>>, // email -> (password_hash/pass, role)
}

impl MemoryAuthProvider {
    pub fn new() -> Self {
        let mut users = HashMap::new();
        users.insert(
            "admin@astrea.local".to_string(),
            ("password123".to_string(), "admin".to_string()),
        );
        users.insert(
            "editor@astrea.local".to_string(),
            ("password123".to_string(), "editor".to_string()),
        );
        users.insert(
            "viewer@astrea.local".to_string(),
            ("password123".to_string(), "viewer".to_string()),
        );
        Self {
            users: Arc::new(RwLock::new(users)),
        }
    }
}

impl Default for MemoryAuthProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuthProvider for MemoryAuthProvider {
    async fn signup(&self, req: &SignupRequest) -> Result<AuthResponse, AppError> {
        let email = req.email.trim().to_lowercase();
        if email.is_empty() || !email.contains('@') {
            return Err(AppError::BadRequest("Valid email is required".into()));
        }
        if req.password.trim().len() < 6 {
            return Err(AppError::BadRequest(
                "Password must be at least 6 characters".into(),
            ));
        }

        let role = req
            .role
            .as_deref()
            .map(str::trim)
            .filter(|r| !r.is_empty())
            .unwrap_or("viewer");

        let sanitized_role = match role.to_lowercase().as_str() {
            "admin" => "admin",
            "editor" | "operator" => "editor",
            _ => "viewer",
        };

        {
            let mut users = self
                .users
                .write()
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;
            if users.contains_key(&email) {
                return Err(AppError::BadRequest(
                    "User with this email already exists".into(),
                ));
            }
            users.insert(
                email.clone(),
                (req.password.clone(), sanitized_role.to_string()),
            );
        }

        let scope = match sanitized_role {
            "admin" => "read:satellites write:satellites admin:satellites",
            "editor" => "read:satellites write:satellites",
            _ => "read:satellites",
        };

        let ttl_seconds = 86400;
        let (token, claims) = create_jwt_token_full(
            &email,
            sanitized_role,
            Some("astrea-sda-api".to_string()),
            Some("astrea-sda-api".to_string()),
            Some(scope.to_string()),
            None,
            ttl_seconds,
        )?;

        Ok(AuthResponse {
            token,
            token_type: "Bearer".to_string(),
            expires_in: ttl_seconds as usize,
            claims,
        })
    }

    async fn login(&self, req: &LoginRequest) -> Result<AuthResponse, AppError> {
        let email = req.email.trim().to_lowercase();
        let (pass, role) = {
            let users = self
                .users
                .read()
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;
            users
                .get(&email)
                .cloned()
                .ok_or_else(|| AppError::Unauthorized("Invalid email or password".into()))?
        };

        if req.password != pass {
            return Err(AppError::Unauthorized("Invalid email or password".into()));
        }

        let scope = match role.as_str() {
            "admin" => "read:satellites write:satellites admin:satellites",
            "editor" => "read:satellites write:satellites",
            _ => "read:satellites",
        };

        let ttl_seconds = 86400;
        let (token, claims) = create_jwt_token_full(
            &email,
            &role,
            Some("astrea-sda-api".to_string()),
            Some("astrea-sda-api".to_string()),
            Some(scope.to_string()),
            None,
            ttl_seconds,
        )?;

        Ok(AuthResponse {
            token,
            token_type: "Bearer".to_string(),
            expires_in: ttl_seconds as usize,
            claims,
        })
    }

    async fn client_assertion_token_exchange(
        &self,
        req: &ClientAssertionRequest,
        client_cert_fingerprint: Option<String>,
    ) -> Result<AuthResponse, AppError> {
        if req.grant_type != "client_credentials"
            && req.grant_type != "urn:ietf:params:oauth:grant-type:jwt-bearer"
        {
            return Err(AppError::BadRequest(
                "Unsupported grant_type. Expected 'client_credentials'".into(),
            ));
        }
        if req.client_assertion_type != "urn:ietf:params:oauth:client-assertion-type:jwt-bearer" {
            return Err(AppError::BadRequest(
                "Unsupported client_assertion_type. Expected 'urn:ietf:params:oauth:client-assertion-type:jwt-bearer'".into(),
            ));
        }

        let assertion_claims = decode_jwt_token(&req.client_assertion)?;
        let client_id = assertion_claims.sub;
        let role = if assertion_claims.role.is_empty() {
            "editor".to_string()
        } else {
            assertion_claims.role
        };

        let scope = req
            .scope
            .clone()
            .or(assertion_claims.scope)
            .unwrap_or_else(|| "read:satellites write:satellites".to_string());

        let cnf = client_cert_fingerprint.map(CnfClaim::new);

        let ttl_seconds = 86400;
        let (token, claims) = create_jwt_token_full(
            &client_id,
            &role,
            Some("astrea-sda-api".to_string()),
            Some("astrea-sda-api".to_string()),
            Some(scope),
            cnf,
            ttl_seconds,
        )?;

        Ok(AuthResponse {
            token,
            token_type: "Bearer".to_string(),
            expires_in: ttl_seconds as usize,
            claims,
        })
    }

    async fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        decode_jwt_token(token)
    }
}

/// Supabase AuthProvider module interacting with Supabase GoTrue Auth REST API.
pub struct SupabaseAuthProvider {
    supabase_url: String,
    anon_key: String,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct SupabaseAuthResponse {
    access_token: String,
    expires_in: Option<usize>,
    user: Option<SupabaseUser>,
}

#[derive(Debug, Deserialize)]
struct SupabaseUser {
    id: String,
    #[allow(dead_code)]
    email: Option<String>,
    user_metadata: Option<serde_json::Value>,
}

impl SupabaseAuthProvider {
    pub fn new(supabase_url: String, anon_key: String) -> Self {
        Self {
            supabase_url: supabase_url.trim_end_matches('/').to_string(),
            anon_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl AuthProvider for SupabaseAuthProvider {
    async fn signup(&self, req: &SignupRequest) -> Result<AuthResponse, AppError> {
        let url = format!("{}/auth/v1/signup", self.supabase_url);
        let role = req
            .role
            .as_deref()
            .map(str::trim)
            .filter(|r| !r.is_empty())
            .unwrap_or("viewer");

        let sanitized_role = match role.to_lowercase().as_str() {
            "admin" => "admin",
            "editor" | "operator" => "editor",
            _ => "viewer",
        };

        let body = serde_json::json!({
            "email": req.email,
            "password": req.password,
            "data": {
                "role": sanitized_role
            }
        });

        let resp = self
            .client
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Supabase auth failed: {}", e)))?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(AppError::BadRequest(format!(
                "Supabase registration failed: {}",
                err_text
            )));
        }

        let auth_res: SupabaseAuthResponse = resp.json().await.map_err(|e| {
            AppError::InternalServerError(format!("Invalid Supabase payload: {}", e))
        })?;

        let user_id = auth_res
            .user
            .as_ref()
            .map(|u| u.id.clone())
            .unwrap_or_else(|| req.email.clone());

        let scope = match sanitized_role {
            "admin" => "read:satellites write:satellites admin:satellites",
            "editor" => "read:satellites write:satellites",
            _ => "read:satellites",
        };

        let ttl_seconds = auth_res.expires_in.unwrap_or(86400);
        let token = auth_res.access_token;
        let claims = Claims {
            sub: user_id,
            iss: Some(self.supabase_url.clone()),
            aud: Some("authenticated".into()),
            exp: chrono::Utc::now().timestamp() as usize + ttl_seconds,
            iat: chrono::Utc::now().timestamp() as usize,
            role: sanitized_role.to_string(),
            roles: Some(vec![sanitized_role.to_string()]),
            scope: Some(scope.to_string()),
            cnf: None,
        };

        Ok(AuthResponse {
            token,
            token_type: "Bearer".to_string(),
            expires_in: ttl_seconds,
            claims,
        })
    }

    async fn login(&self, req: &LoginRequest) -> Result<AuthResponse, AppError> {
        let url = format!("{}/auth/v1/token?grant_type=password", self.supabase_url);

        let body = serde_json::json!({
            "email": req.email,
            "password": req.password
        });

        let resp = self
            .client
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Supabase login failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::Unauthorized(
                "Invalid Supabase email or password".into(),
            ));
        }

        let auth_res: SupabaseAuthResponse = resp.json().await.map_err(|e| {
            AppError::InternalServerError(format!("Invalid Supabase payload: {}", e))
        })?;

        let (user_id, role) = if let Some(ref u) = auth_res.user {
            let r = u
                .user_metadata
                .as_ref()
                .and_then(|m| m.get("role"))
                .and_then(|v| v.as_str())
                .unwrap_or("viewer");
            (u.id.clone(), r.to_string())
        } else {
            (req.email.clone(), "viewer".to_string())
        };

        let ttl_seconds = auth_res.expires_in.unwrap_or(86400);
        let claims = Claims {
            sub: user_id,
            iss: Some(self.supabase_url.clone()),
            aud: Some("authenticated".into()),
            exp: chrono::Utc::now().timestamp() as usize + ttl_seconds,
            iat: chrono::Utc::now().timestamp() as usize,
            role: role.clone(),
            roles: Some(vec![role]),
            scope: Some("read:satellites".into()),
            cnf: None,
        };

        Ok(AuthResponse {
            token: auth_res.access_token,
            token_type: "Bearer".to_string(),
            expires_in: ttl_seconds,
            claims,
        })
    }

    async fn client_assertion_token_exchange(
        &self,
        req: &ClientAssertionRequest,
        client_cert_fingerprint: Option<String>,
    ) -> Result<AuthResponse, AppError> {
        if req.grant_type != "client_credentials"
            && req.grant_type != "urn:ietf:params:oauth:grant-type:jwt-bearer"
        {
            return Err(AppError::BadRequest(
                "Unsupported grant_type. Expected 'client_credentials'".into(),
            ));
        }
        if req.client_assertion_type != "urn:ietf:params:oauth:client-assertion-type:jwt-bearer" {
            return Err(AppError::BadRequest(
                "Unsupported client_assertion_type. Expected 'urn:ietf:params:oauth:client-assertion-type:jwt-bearer'".into(),
            ));
        }

        let assertion_claims = decode_jwt_token(&req.client_assertion)?;
        let client_id = assertion_claims.sub;
        let role = if assertion_claims.role.is_empty() {
            "editor".to_string()
        } else {
            assertion_claims.role
        };

        let scope = req
            .scope
            .clone()
            .or(assertion_claims.scope)
            .unwrap_or_else(|| "read:satellites write:satellites".to_string());

        let cnf = client_cert_fingerprint.map(CnfClaim::new);

        let ttl_seconds = 86400;
        let (token, claims) = create_jwt_token_full(
            &client_id,
            &role,
            Some(self.supabase_url.clone()),
            Some("authenticated".to_string()),
            Some(scope),
            cnf,
            ttl_seconds,
        )?;

        Ok(AuthResponse {
            token,
            token_type: "Bearer".to_string(),
            expires_in: ttl_seconds as usize,
            claims,
        })
    }

    async fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        decode_jwt_token(token)
    }
}

/// Native PostgreSQL AuthProvider storing developer credentials in database `users` table.
pub struct PostgresAuthProvider {
    pool: sqlx::PgPool,
}

impl PostgresAuthProvider {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthProvider for PostgresAuthProvider {
    async fn signup(&self, req: &SignupRequest) -> Result<AuthResponse, AppError> {
        let email = req.email.trim().to_lowercase();
        if email.is_empty() || !email.contains('@') {
            return Err(AppError::BadRequest("Valid email is required".into()));
        }
        if req.password.trim().len() < 6 {
            return Err(AppError::BadRequest(
                "Password must be at least 6 characters".into(),
            ));
        }

        let role = req
            .role
            .as_deref()
            .map(str::trim)
            .filter(|r| !r.is_empty())
            .unwrap_or("viewer");

        let sanitized_role = match role.to_lowercase().as_str() {
            "admin" => "admin",
            "editor" | "operator" => "editor",
            _ => "viewer",
        };

        let user_id = uuid::Uuid::new_v4();
        let now = chrono::Utc::now();

        let insert_res = sqlx::query(
            "INSERT INTO users (id, email, password_hash, role, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(user_id)
        .bind(&email)
        .bind(&req.password) // In production, hash with argon2/bcrypt; plain/salted string for test DB
        .bind(sanitized_role)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await;

        if let Err(e) = insert_res {
            return Err(AppError::BadRequest(format!(
                "Failed to register user: {}",
                e
            )));
        }

        let scope = match sanitized_role {
            "admin" => "read:satellites write:satellites admin:satellites",
            "editor" => "read:satellites write:satellites",
            _ => "read:satellites",
        };

        let ttl_seconds = 86400;
        let (token, claims) = create_jwt_token_full(
            &email,
            sanitized_role,
            Some("astrea-sda-api".to_string()),
            Some("astrea-sda-api".to_string()),
            Some(scope.to_string()),
            None,
            ttl_seconds,
        )?;

        Ok(AuthResponse {
            token,
            token_type: "Bearer".to_string(),
            expires_in: ttl_seconds as usize,
            claims,
        })
    }

    async fn login(&self, req: &LoginRequest) -> Result<AuthResponse, AppError> {
        let email = req.email.trim().to_lowercase();
        let row =
            sqlx::query("SELECT email, password_hash, role FROM users WHERE LOWER(email) = $1")
                .bind(&email)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let user = row.ok_or_else(|| AppError::Unauthorized("Invalid email or password".into()))?;
        let db_email: String = user
            .try_get("email")
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let password_hash: String = user
            .try_get("password_hash")
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let role: String = user
            .try_get("role")
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        if req.password != password_hash {
            return Err(AppError::Unauthorized("Invalid email or password".into()));
        }

        let scope = match role.as_str() {
            "admin" => "read:satellites write:satellites admin:satellites",
            "editor" => "read:satellites write:satellites",
            _ => "read:satellites",
        };

        let ttl_seconds = 86400;
        let (token, claims) = create_jwt_token_full(
            &db_email,
            &role,
            Some("astrea-sda-api".to_string()),
            Some("astrea-sda-api".to_string()),
            Some(scope.to_string()),
            None,
            ttl_seconds,
        )?;

        Ok(AuthResponse {
            token,
            token_type: "Bearer".to_string(),
            expires_in: ttl_seconds as usize,
            claims,
        })
    }

    async fn client_assertion_token_exchange(
        &self,
        req: &ClientAssertionRequest,
        client_cert_fingerprint: Option<String>,
    ) -> Result<AuthResponse, AppError> {
        if req.grant_type != "client_credentials"
            && req.grant_type != "urn:ietf:params:oauth:grant-type:jwt-bearer"
        {
            return Err(AppError::BadRequest(
                "Unsupported grant_type. Expected 'client_credentials'".into(),
            ));
        }
        if req.client_assertion_type != "urn:ietf:params:oauth:client-assertion-type:jwt-bearer" {
            return Err(AppError::BadRequest(
                "Unsupported client_assertion_type. Expected 'urn:ietf:params:oauth:client-assertion-type:jwt-bearer'".into(),
            ));
        }

        let assertion_claims = decode_jwt_token(&req.client_assertion)?;
        let client_id = assertion_claims.sub;
        let role = if assertion_claims.role.is_empty() {
            "editor".to_string()
        } else {
            assertion_claims.role
        };

        let scope = req
            .scope
            .clone()
            .or(assertion_claims.scope)
            .unwrap_or_else(|| "read:satellites write:satellites".to_string());

        let cnf = client_cert_fingerprint.map(CnfClaim::new);

        let ttl_seconds = 86400;
        let (token, claims) = create_jwt_token_full(
            &client_id,
            &role,
            Some("astrea-sda-api".to_string()),
            Some("astrea-sda-api".to_string()),
            Some(scope),
            cnf,
            ttl_seconds,
        )?;

        Ok(AuthResponse {
            token,
            token_type: "Bearer".to_string(),
            expires_in: ttl_seconds as usize,
            claims,
        })
    }

    async fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        decode_jwt_token(token)
    }
}
