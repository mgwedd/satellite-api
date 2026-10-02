//! Modular Authentication Architecture
//!
//! Provides clean separation of concern across authentication subsystems:
//! - `claims`: JWT Claims payload, RBAC hierarchy (`Viewer`, `Editor`, `Admin`), and Axum `FromRequestParts` extractor.
//! - `keys`: RSA key loading (Production Secret Manager/Vault vs Local Dev file fallbacks).
//! - `jwt`: RS256 JWT encoding, decoding, and signature validation.
//! - `mtls`: RFC 8705 Mutual TLS (mTLS) certificate fingerprint verification & token binding (`cnf` claim).
//! - `provider`: `AuthProvider` trait with `MemoryAuthProvider`, `PostgresAuthProvider`, and `SupabaseAuthProvider`.
//! - `handlers`: Axum HTTP handlers (`/v1/auth/signup`, `/v1/auth/login`, `/v1/auth/token`, `/v1/auth/me`).

pub mod claims;
pub mod dto;
pub mod handlers;
pub mod jwt;
pub mod keys;
pub mod mtls;
pub mod provider;

pub use claims::*;
pub use dto::*;
pub use handlers::*;
pub use jwt::*;
pub use keys::*;
pub use mtls::*;
pub use provider::*;
