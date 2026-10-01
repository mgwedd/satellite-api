use crate::auth::{Claims, UserRole};
use crate::error::AppError;
use crate::models::{
    CreateSatelliteDto, GroundTrackResponse, IlluminationResponse, NextVisiblePassResponse,
    OverheadResponse, Satellite, UpdateSatelliteDto,
};
use crate::pagination::{PaginatedResponse, PaginationQuery};
use crate::repository::SatelliteRepository;
use crate::services::astrodynamics;
use crate::services::pipeline::{CelesTrakGroup, DiscoveryPipeline};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Deserialize, IntoParams)]
pub struct OverheadQueryParams {
    /// Observer latitude in decimal degrees (-90.0 to 90.0)
    pub lat: f64,
    /// Observer longitude in decimal degrees (-180.0 to 180.0)
    pub lon: f64,
    /// Observer altitude above sea level in meters
    pub alt: Option<f64>,
    /// UTC timestamp for calculation epoch (defaults to current time if omitted)
    pub time: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct NextVisibleQueryParams {
    /// Observer latitude in decimal degrees (-90.0 to 90.0)
    pub lat: f64,
    /// Observer longitude in decimal degrees (-180.0 to 180.0)
    pub lon: f64,
    /// Observer altitude above sea level in meters
    pub alt: Option<f64>,
    /// Minimum elevation angle threshold in degrees (default: 5.0 deg)
    pub threshold_deg: Option<f64>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct GroundTrackQueryParams {
    /// Trajectory projection duration in minutes (default: 90 mins, max: 1440)
    pub duration_minutes: Option<usize>,
    /// Step sampling interval in seconds (default: 30 secs, max: 300)
    pub step_seconds: Option<usize>,
    /// Output format ('geojson' or 'json', default: 'geojson')
    pub format: Option<String>,
    /// UTC timestamp to start projection from (defaults to current time if omitted)
    pub start_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct PipelineSyncQueryParams {
    /// CelesTrak satellite group name (e.g. 'stations', 'visual', 'starlink', 'weather', 'active', 'last-30-days')
    pub group: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PipelineSyncResponse {
    pub group: String,
    pub synced_count: usize,
    pub message: String,
}

/// Create Satellite
///
/// Creates a new satellite record from Two-Line Element (TLE) set data. Protected by JWT auth (requires 'editor' or 'admin' role).
#[utoipa::path(
    post,
    path = "/v1/satellites",
    operation_id = "createSatellite",
    request_body = CreateSatelliteDto,
    responses(
        (status = 201, description = "Satellite created successfully", body = Satellite),
        (status = 400, description = "Invalid request payload", body = ErrorResponse),
        (status = 401, description = "Unauthorized - Missing or invalid JWT token", body = ErrorResponse),
        (status = 403, description = "Forbidden - Insufficient role permissions", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "Satellites"
)]
pub async fn create_satellite(
    claims: Claims,
    State(repo): State<SatelliteRepository>,
    Json(dto): Json<CreateSatelliteDto>,
) -> Result<(StatusCode, Json<Satellite>), AppError> {
    claims.require_role(UserRole::Editor)?;
    tracing::info!("Satellite creation requested by JWT user: {}", claims.sub);
    let satellite = repo.create_satellite(dto).await?;
    Ok((StatusCode::CREATED, Json(satellite)))
}

/// List Satellites
///
/// Retrieves a paginated list of satellites using base64 checkpoint cursors.
#[utoipa::path(
    get,
    path = "/v1/satellites",
    operation_id = "listSatellites",
    params(PaginationQuery),
    responses(
        (status = 200, description = "Paginated list of satellites", body = PaginatedResponseSatellite)
    ),
    tag = "Satellites"
)]
pub async fn list_satellites(
    State(repo): State<SatelliteRepository>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<Satellite>>, AppError> {
    let paginated_res = repo.list_satellites_paginated(pagination).await?;
    Ok(Json(paginated_res))
}

/// Get Satellite by ID
///
/// Retrieves details for a specific satellite by its unique UUID.
#[utoipa::path(
    get,
    path = "/v1/satellites/{id}",
    operation_id = "getSatellite",
    params(
        ("id" = Uuid, Path, description = "Satellite unique UUID identifier")
    ),
    responses(
        (status = 200, description = "Satellite found", body = Satellite),
        (status = 404, description = "Satellite not found", body = ErrorResponse)
    ),
    tag = "Satellites"
)]
pub async fn get_satellite(
    State(repo): State<SatelliteRepository>,
    Path(id): Path<Uuid>,
) -> Result<Json<Satellite>, AppError> {
    let satellite = repo.get_satellite_by_id(id).await?;
    Ok(Json(satellite))
}

/// Update Satellite
///
/// Updates a satellite's name or TLE orbital parameters. Protected by JWT auth (requires 'editor' or 'admin' role).
#[utoipa::path(
    patch,
    path = "/v1/satellites/{id}",
    operation_id = "updateSatellite",
    params(
        ("id" = Uuid, Path, description = "Satellite unique UUID identifier")
    ),
    request_body = UpdateSatelliteDto,
    responses(
        (status = 200, description = "Satellite updated successfully", body = Satellite),
        (status = 401, description = "Unauthorized - Missing or invalid JWT token", body = ErrorResponse),
        (status = 403, description = "Forbidden - Insufficient role permissions", body = ErrorResponse),
        (status = 404, description = "Satellite not found", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "Satellites"
)]
pub async fn update_satellite(
    claims: Claims,
    State(repo): State<SatelliteRepository>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateSatelliteDto>,
) -> Result<Json<Satellite>, AppError> {
    claims.require_role(UserRole::Editor)?;
    tracing::info!("Satellite update requested by JWT user: {}", claims.sub);
    let satellite = repo.update_satellite_by_id(id, dto).await?;
    Ok(Json(satellite))
}

/// Delete Satellite
///
/// Deletes a satellite record by its unique UUID. Protected by JWT auth (requires 'admin' role).
#[utoipa::path(
    delete,
    path = "/v1/satellites/{id}",
    operation_id = "deleteSatellite",
    params(
        ("id" = Uuid, Path, description = "Satellite unique UUID identifier")
    ),
    responses(
        (status = 204, description = "Satellite deleted successfully"),
        (status = 401, description = "Unauthorized - Missing or invalid JWT token", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin role required", body = ErrorResponse),
        (status = 404, description = "Satellite not found", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "Satellites"
)]
pub async fn delete_satellite(
    claims: Claims,
    State(repo): State<SatelliteRepository>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    claims.require_role(UserRole::Admin)?;
    tracing::info!("Satellite deletion requested by JWT admin: {}", claims.sub);
    repo.delete_satellite_by_id(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Trigger CelesTrak Pipeline Sync
///
/// Triggers automated CelesTrak discovery pipeline synchronization for a specific satellite group. Protected by JWT auth (requires 'editor' or 'admin' role).
#[utoipa::path(
    post,
    path = "/v1/pipelines/sync",
    operation_id = "triggerPipelineSync",
    params(PipelineSyncQueryParams),
    responses(
        (status = 200, description = "Pipeline sync completed successfully", body = PipelineSyncResponse),
        (status = 401, description = "Unauthorized - Missing or invalid JWT token", body = ErrorResponse),
        (status = 403, description = "Forbidden - Insufficient role permissions", body = ErrorResponse),
        (status = 500, description = "Pipeline sync execution failed", body = ErrorResponse)
    ),
    security(("bearer_auth" = [])),
    tag = "Pipelines"
)]
pub async fn trigger_pipeline_sync(
    claims: Claims,
    State(repo): State<SatelliteRepository>,
    Query(params): Query<PipelineSyncQueryParams>,
) -> Result<Json<PipelineSyncResponse>, AppError> {
    claims.require_role(UserRole::Editor)?;
    tracing::info!("Pipeline sync triggered by JWT user: {}", claims.sub);

    let group = match params.group.as_deref() {
        Some("visual") => CelesTrakGroup::Visual,
        Some("starlink") => CelesTrakGroup::Starlink,
        Some("weather") => CelesTrakGroup::Weather,
        Some("last-30-days") => CelesTrakGroup::Last30Days,
        Some("active") => CelesTrakGroup::Active,
        _ => CelesTrakGroup::Stations,
    };

    let group_name = group.as_str().to_string();

    let synced_count = DiscoveryPipeline::sync_group(&repo, group)
        .await
        .map_err(AppError::InternalServerError)?;

    Ok(Json(PipelineSyncResponse {
        group: group_name.clone(),
        synced_count,
        message: format!(
            "Successfully synced {} satellites for group '{}'",
            synced_count, group_name
        ),
    }))
}

/// Get Overhead Satellite
///
/// Computes the satellite closest to overhead (highest elevation) across all tracked satellites using parallel Rayon propagation.
#[utoipa::path(
    get,
    path = "/v1/astrodynamics/overhead",
    operation_id = "getOverheadSatellite",
    params(OverheadQueryParams),
    responses(
        (status = 200, description = "Overhead satellite computed successfully", body = OverheadResponse),
        (status = 404, description = "No satellite overhead found", body = ErrorResponse)
    ),
    tag = "Astrodynamics"
)]
pub async fn get_overhead(
    State(repo): State<SatelliteRepository>,
    Query(params): Query<OverheadQueryParams>,
) -> Result<Json<OverheadResponse>, AppError> {
    let satellites = repo.list_satellites().await?;
    if satellites.is_empty() {
        return Err(AppError::NotFound);
    }

    let time = params.time.unwrap_or_else(Utc::now);
    let alt = params.alt.unwrap_or(0.0);
    let time_bucket = time.timestamp() / 10; // Round time to 10-second buckets for cache reuse

    let cache_key = format!(
        "overhead:{:.2}:{:.2}:{:.1}:{}",
        params.lat, params.lon, alt, time_bucket
    );

    let res = repo
        .cache
        .get_or_insert_with(&cache_key, || async move {
            let overhead_res = tokio::task::spawn_blocking(move || {
                astrodynamics::find_overhead_satellite(
                    &satellites,
                    params.lat,
                    params.lon,
                    alt,
                    time,
                )
            })
            .await
            .map_err(|e| e.to_string())?;

            overhead_res.ok_or_else(|| "No satellite overhead found".to_string())
        })
        .await
        .map_err(|e| {
            if e == "No satellite overhead found" {
                AppError::NotFound
            } else {
                AppError::InternalServerError(e)
            }
        })?;

    Ok(Json(res))
}

/// Get Next Visible Pass
///
/// Calculates the next visible pass for a specific satellite above elevation threshold.
#[utoipa::path(
    get,
    path = "/v1/satellites/{id}/next-visible",
    operation_id = "getNextVisiblePass",
    params(
        ("id" = Uuid, Path, description = "Satellite unique UUID identifier"),
        NextVisibleQueryParams
    ),
    responses(
        (status = 200, description = "Next visible pass calculated successfully", body = NextVisiblePassResponse),
        (status = 404, description = "Satellite not found", body = ErrorResponse)
    ),
    tag = "Astrodynamics"
)]
pub async fn get_next_visible(
    State(repo): State<SatelliteRepository>,
    Path(id): Path<Uuid>,
    Query(params): Query<NextVisibleQueryParams>,
) -> Result<Json<NextVisiblePassResponse>, AppError> {
    let satellite = repo.get_satellite_by_id(id).await?;
    let start_time = Utc::now();
    let alt = params.alt.unwrap_or(0.0);
    let threshold = params.threshold_deg.unwrap_or(5.0);
    let time_bucket = start_time.timestamp() / 60; // Round start time to 1-minute bucket for cache reuse

    let cache_key = format!(
        "next_visible:{}:{:.2}:{:.2}:{:.1}:{:.1}:{}",
        id, params.lat, params.lon, alt, threshold, time_bucket
    );

    let res = repo
        .cache
        .get_or_insert_with(&cache_key, || async move {
            let pass_res = tokio::task::spawn_blocking(move || {
                astrodynamics::find_next_visible_pass(
                    &satellite, params.lat, params.lon, alt, start_time, threshold, 1440,
                )
            })
            .await
            .map_err(|e| e.to_string())?;

            pass_res.map_err(|e| e.to_string())
        })
        .await
        .map_err(AppError::InternalServerError)?;

    Ok(Json(res))
}

/// Get Satellite 3D Ground Track & Trajectory
///
/// Computes 3D ECF coordinates, geodetic position, orbital period, footprint radius, and GeoJSON ground track line.
#[utoipa::path(
    get,
    path = "/v1/satellites/{id}/groundtrack",
    operation_id = "getGroundTrack",
    params(
        ("id" = Uuid, Path, description = "Satellite UUID"),
        GroundTrackQueryParams,
    ),
    responses(
        (status = 200, description = "3D Ground Track and GeoJSON trajectory", body = GroundTrackResponse),
        (status = 404, description = "Satellite not found", body = ErrorResponse),
        (status = 500, description = "Internal calculation error", body = ErrorResponse)
    ),
    tag = "Astrodynamics"
)]
pub async fn get_ground_track(
    Path(id): Path<Uuid>,
    Query(params): Query<GroundTrackQueryParams>,
    State(repo): State<SatelliteRepository>,
) -> Result<Json<GroundTrackResponse>, AppError> {
    let satellite = repo.get_satellite_by_id(id).await?;
    let start_time = params.start_time.unwrap_or_else(Utc::now);
    let duration_minutes = params.duration_minutes.unwrap_or(90);
    let step_seconds = params.step_seconds.unwrap_or(30);
    let (include_geojson, include_czml) = match params.format.as_deref() {
        Some(f) if f.eq_ignore_ascii_case("czml") => (false, true),
        Some(f) if f.eq_ignore_ascii_case("json") => (false, false),
        Some(f) if f.eq_ignore_ascii_case("all") => (true, true),
        _ => (true, false),
    };

    let time_bucket = start_time.timestamp() / 60;
    let cache_key = format!(
        "groundtrack:{}:{}:{}:{}:{}:{}:{}",
        id,
        duration_minutes,
        step_seconds,
        include_geojson,
        include_czml,
        time_bucket,
        satellite.last_modified_date.timestamp()
    );

    let res = repo
        .cache
        .get_or_insert_with(&cache_key, || async move {
            let sat_clone = satellite.clone();
            let track_res = tokio::task::spawn_blocking(move || {
                astrodynamics::generate_ground_track(
                    &sat_clone,
                    start_time,
                    duration_minutes,
                    step_seconds,
                    include_geojson,
                    include_czml,
                )
            })
            .await
            .map_err(|e| e.to_string())?;

            track_res.map_err(|e| e.to_string())
        })
        .await
        .map_err(AppError::InternalServerError)?;

    Ok(Json(res))
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct IlluminationQueryParams {
    /// Observer latitude in decimal degrees (-90.0 to 90.0)
    pub lat: f64,
    /// Observer longitude in decimal degrees (-180.0 to 180.0)
    pub lon: f64,
    /// Observer altitude above sea level in meters (default: 0.0 m)
    pub alt: Option<f64>,
    /// UTC timestamp for calculation epoch (defaults to current time if omitted)
    pub time: Option<DateTime<Utc>>,
}

/// Get Satellite Illumination & Visual Magnitude
///
/// Computes solar shadow geometry (FullSunlight, Penumbra, Umbra), observer twilight state, observable status, and visual magnitude.
#[utoipa::path(
    get,
    path = "/v1/satellites/{id}/illumination",
    operation_id = "getSatelliteIllumination",
    params(
        ("id" = Uuid, Path, description = "Satellite unique UUID identifier"),
        IlluminationQueryParams
    ),
    responses(
        (status = 200, description = "Satellite illumination status computed successfully", body = IlluminationResponse),
        (status = 404, description = "Satellite not found", body = ErrorResponse)
    ),
    tag = "Astrodynamics"
)]
pub async fn get_satellite_illumination(
    State(repo): State<SatelliteRepository>,
    Path(id): Path<Uuid>,
    Query(params): Query<IlluminationQueryParams>,
) -> Result<Json<IlluminationResponse>, AppError> {
    let satellite = repo.get_satellite_by_id(id).await?;
    let time = params.time.unwrap_or_else(Utc::now);
    let alt = params.alt.unwrap_or(0.0);

    let time_bucket = time.timestamp() / 10;
    let cache_key = format!(
        "illumination:{}:{:.2}:{:.2}:{:.1}:{}",
        id, params.lat, params.lon, alt, time_bucket
    );

    let res = repo
        .cache
        .get_or_insert_with(&cache_key, || async move {
            let sat_clone = satellite.clone();
            let illum_res = tokio::task::spawn_blocking(move || {
                astrodynamics::calculate_illumination(&sat_clone, params.lat, params.lon, alt, time)
            })
            .await
            .map_err(|e| e.to_string())?;

            illum_res.map_err(|e| e.to_string())
        })
        .await
        .map_err(AppError::InternalServerError)?;

    Ok(Json(res))
}
