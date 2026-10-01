use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub enum TransitTarget {
    Sun,
    Moon,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TransitMatch {
    pub satellite_id: Uuid,
    pub satellite_name: String,
    pub transit_start_utc: DateTime<Utc>,
    pub transit_center_utc: DateTime<Utc>,
    pub transit_end_utc: DateTime<Utc>,
    pub transit_duration_seconds: f64,
    pub min_angular_separation_deg: f64,
    pub target_elevation_deg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TransitPredictionResponse {
    pub target: TransitTarget,
    pub observer_lat: f64,
    pub observer_lon: f64,
    pub forecast_days: usize,
    pub transits_found: usize,
    pub results: Vec<TransitMatch>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct TransitQueryParams {
    /// Observer latitude in decimal degrees (-90.0 to 90.0)
    pub lat: f64,
    /// Observer longitude in decimal degrees (-180.0 to 180.0)
    pub lon: f64,
    /// Observer altitude above sea level in meters (default: 0.0)
    pub alt: Option<f64>,
    /// Forecast window in days (default: 7 days, max: 30)
    pub duration_days: Option<usize>,
    /// Maximum angular separation from Sun/Moon center in degrees (default: 0.5 degrees)
    pub max_angular_separation_deg: Option<f64>,
}
