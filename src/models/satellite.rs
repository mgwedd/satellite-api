use crate::pagination::IdentifiableCheckpoint;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Tle {
    pub line_one: String,
    pub line_two: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Satellite {
    pub id: Uuid,
    pub name: String,
    pub tle: Tle,
    pub created_date: DateTime<Utc>,
    pub last_modified_date: DateTime<Utc>,
}

impl IdentifiableCheckpoint for Satellite {
    fn checkpoint_id(&self) -> Uuid {
        self.id
    }

    fn checkpoint_timestamp(&self) -> i64 {
        self.created_date.timestamp_millis()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateSatelliteDto {
    pub name: String,
    #[serde(alias = "tleLineOne")]
    pub line_one: String,
    #[serde(alias = "tleLineTwo")]
    pub line_two: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSatelliteDto {
    pub name: Option<String>,
    #[serde(alias = "tleLineOne")]
    pub line_one: Option<String>,
    #[serde(alias = "tleLineTwo")]
    pub line_two: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GroundPositionQuery {
    pub lat: f64,
    pub lon: f64,
    pub alt: Option<f64>,
    pub time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OverheadResponse {
    pub satellite: Satellite,
    pub elevation: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NextVisiblePassResponse {
    pub satellite_id: Uuid,
    pub satellite_name: String,
    pub pass_time: DateTime<Utc>,
    pub elevation_deg: f64,
    pub azimuth_deg: f64,
    pub range_km: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GroundTrackPoint {
    pub timestamp: DateTime<Utc>,
    pub lat: f64,
    pub lon: f64,
    pub alt_km: f64,
    pub position_ecf_km: [f64; 3],
    pub velocity_ecf_kms: [f64; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeoJsonGeometry {
    pub r#type: String,
    pub coordinates: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeoJsonFeature {
    pub r#type: String,
    pub geometry: GeoJsonGeometry,
    pub properties: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GroundTrackResponse {
    pub satellite_id: Uuid,
    pub satellite_name: String,
    pub orbital_period_minutes: f64,
    pub footprint_radius_km: f64,
    pub duration_minutes: usize,
    pub step_seconds: usize,
    pub trajectory: Vec<GroundTrackPoint>,
    pub geojson: Option<GeoJsonFeature>,
    pub footprint_polygon: Option<GeoJsonFeature>,
    pub czml: Option<serde_json::Value>,
}
