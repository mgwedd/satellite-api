use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

/// Observed element change between two consecutive TLEs. Describes what changed, not why:
/// burn intent (stationkeeping, collision avoidance, ...) is not observable from TLEs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub enum ManeuverType {
    SemiMajorAxisIncrease,
    SemiMajorAxisDecrease,
    InclinationChange,
    Combined,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub enum AnomalyStatus {
    Nominal,
    AnomalyDetected,
    /// Fewer residuals than needed for robust statistics; no verdict is given.
    InsufficientData,
}

/// A candidate maneuver: an element discontinuity between two consecutive TLEs that
/// exceeds the requested thresholds. It happened somewhere inside the window; TLEs
/// cannot localise it further. Unmodelled drag or TLE fit errors can also cause one.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DetectedManeuver {
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    /// Observed mean semi-major axis minus the value predicted by the earlier TLE's
    /// mean-motion derivatives, in km
    pub semi_major_axis_residual_km: f64,
    /// Observed mean inclination change, in degrees
    pub inclination_change_deg: f64,
    pub maneuver_type: ManeuverType,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ManeuversResponse {
    pub satellite_id: Uuid,
    pub satellite_name: String,
    pub tle_epochs_analyzed: usize,
    pub total_maneuvers_detected: usize,
    pub maneuvers: Vec<DetectedManeuver>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct ManeuverQueryParams {
    /// Only report windows ending at or after this time
    pub start_time: Option<DateTime<Utc>>,
    /// Only report windows ending at or before this time
    pub end_time: Option<DateTime<Utc>>,
    /// Minimum |semi-major axis residual| in km (default 0.1)
    pub min_sma_change_km: Option<f64>,
    /// Minimum |inclination change| in degrees (default 0.005)
    pub min_inclination_change_deg: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnomalyDetectionRequest {
    /// Robust z-score (median/MAD) threshold (default 3.0)
    pub threshold_sigma: Option<f64>,
    pub min_sma_change_km: Option<f64>,
    pub min_inclination_change_deg: Option<f64>,
}

/// Mean elements of one TLE and, for every TLE after the first, its change from the previous.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrbitalParameterResidual {
    pub epoch: DateTime<Utc>,
    pub mean_semi_major_axis_km: f64,
    pub inclination_deg: f64,
    pub mean_motion_revday: f64,
    pub eccentricity: f64,
    pub bstar_drag: f64,
    pub semi_major_axis_residual_km: Option<f64>,
    pub inclination_change_deg: Option<f64>,
    pub is_anomaly: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnomalyDetectionResponse {
    pub satellite_id: Uuid,
    pub satellite_name: String,
    pub status: AnomalyStatus,
    pub anomalies_detected: usize,
    pub residual_history: Vec<OrbitalParameterResidual>,
}
