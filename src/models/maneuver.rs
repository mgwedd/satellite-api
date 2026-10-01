use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub enum ManeuverType {
    OrbitRaising,
    InclinationChange,
    Stationkeeping,
    CollisionAvoidance,
    DeorbitBurn,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub enum AnomalySeverity {
    Nominal,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeltaVComponents {
    /// Radial component of delta-V vector in m/s (along position vector)
    pub radial_ms: f64,
    /// Tangential / along-track component of delta-V vector in m/s (along velocity direction)
    pub tangential_ms: f64,
    /// Normal / cross-track component of delta-V vector in m/s (out of orbital plane)
    pub normal_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DetectedManeuver {
    pub maneuver_id: Uuid,
    pub satellite_id: Uuid,
    pub satellite_name: String,
    pub detected_at_epoch: DateTime<Utc>,
    pub total_delta_v_ms: f64,
    pub delta_v_components: DeltaVComponents,
    pub maneuver_type: ManeuverType,
    pub semi_major_axis_change_km: f64,
    pub inclination_change_deg: f64,
    pub estimated_fuel_used_kg: Option<f64>,
    pub confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ManeuversResponse {
    pub satellite_id: Uuid,
    pub satellite_name: String,
    pub total_maneuvers_detected: usize,
    pub cumulative_delta_v_ms: f64,
    pub maneuvers: Vec<DetectedManeuver>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct ManeuverQueryParams {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub min_delta_v_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnomalyDetectionRequest {
    pub threshold_sigma: Option<f64>,
    pub min_sma_change_km: Option<f64>,
    pub min_inclination_change_deg: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrbitalParameterResidual {
    pub epoch: DateTime<Utc>,
    pub semi_major_axis_km: f64,
    pub inclination_deg: f64,
    pub mean_motion_revday: f64,
    pub eccentricity: f64,
    pub bstar_drag: f64,
    pub delta_semi_major_axis_km: f64,
    pub delta_inclination_deg: f64,
    pub is_anomaly: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnomalyDetectionResponse {
    pub satellite_id: Uuid,
    pub satellite_name: String,
    pub severity: AnomalySeverity,
    pub anomalies_detected: usize,
    pub max_residual_sigma: f64,
    pub residual_history: Vec<OrbitalParameterResidual>,
    pub recommendation: String,
}
