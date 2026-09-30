use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

/// Query parameters for paginated endpoints
#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct PaginationQuery {
    pub limit: Option<usize>,
    pub cursor: Option<String>,
}

impl PaginationQuery {
    pub fn limit(&self, default_limit: usize, max_limit: usize) -> usize {
        self.limit
            .unwrap_or(default_limit)
            .min(max_limit)
            .max(1)
    }
}

/// Opaque checkpoint cursor data structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct CheckpointCursor {
    pub last_id: Uuid,
    pub checkpoint_timestamp: i64,
}

impl CheckpointCursor {
    pub fn new(last_id: Uuid, checkpoint_timestamp: i64) -> Self {
        Self {
            last_id,
            checkpoint_timestamp,
        }
    }

    /// Encodes cursor into an opaque base64 string
    pub fn encode(&self) -> Result<String, String> {
        let json_str = serde_json::to_string(self).map_err(|e| e.to_string())?;
        Ok(URL_SAFE_NO_PAD.encode(json_str.as_bytes()))
    }

    /// Decodes an opaque base64 string into a CheckpointCursor
    pub fn decode(encoded: &str) -> Result<Self, String> {
        let decoded_bytes = URL_SAFE_NO_PAD
            .decode(encoded.as_bytes())
            .map_err(|e| format!("Invalid cursor encoding: {}", e))?;
        let json_str = String::from_utf8(decoded_bytes)
            .map_err(|e| format!("Invalid cursor utf8 string: {}", e))?;
        serde_json::from_str(&json_str).map_err(|e| format!("Invalid cursor JSON: {}", e))
    }
}

/// Metadata envelope for paginated responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaginationMeta {
    pub next_cursor: Option<String>,
    pub has_more: bool,
    pub limit: usize,
    pub total_count: usize,
}

use crate::models::Satellite;

/// Generic Paginated Response envelope
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[aliases(PaginatedResponseSatellite = PaginatedResponse<Satellite>)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

pub trait IdentifiableCheckpoint {
    fn checkpoint_id(&self) -> Uuid;
    fn checkpoint_timestamp(&self) -> i64;
}
