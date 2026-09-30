use crate::error::AppError;
use crate::models::{
    CreateSatelliteDto, NextVisiblePassResponse, OverheadResponse, Satellite, UpdateSatelliteDto,
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
use uuid::Uuid;

use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams)]
pub struct OverheadQueryParams {
    pub lat: f64,
    pub lon: f64,
    pub alt: Option<f64>,
    pub time: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct NextVisibleQueryParams {
    pub lat: f64,
    pub lon: f64,
    pub alt: Option<f64>,
    pub threshold_deg: Option<f64>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct PipelineSyncQueryParams {
    pub group: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PipelineSyncResponse {
    pub group: String,
    pub synced_count: usize,
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/v1/satellites",
    request_body = CreateSatelliteDto,
    responses(
        (status = 201, description = "Satellite created successfully", body = Satellite),
        (status = 400, description = "Invalid request payload")
    ),
    tag = "Satellites"
)]
pub async fn create_satellite(
    State(repo): State<SatelliteRepository>,
    Json(dto): Json<CreateSatelliteDto>,
) -> Result<(StatusCode, Json<Satellite>), AppError> {
    let satellite = repo.create_satellite(dto).await?;
    Ok((StatusCode::CREATED, Json(satellite)))
}

#[utoipa::path(
    get,
    path = "/v1/satellites",
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

#[utoipa::path(
    get,
    path = "/v1/satellites/{id}",
    params(
        ("id" = Uuid, Path, description = "Satellite unique UUID identifier")
    ),
    responses(
        (status = 200, description = "Satellite found", body = Satellite),
        (status = 404, description = "Satellite not found")
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

#[utoipa::path(
    patch,
    path = "/v1/satellites/{id}",
    params(
        ("id" = Uuid, Path, description = "Satellite unique UUID identifier")
    ),
    request_body = UpdateSatelliteDto,
    responses(
        (status = 200, description = "Satellite updated successfully", body = Satellite),
        (status = 404, description = "Satellite not found")
    ),
    tag = "Satellites"
)]
pub async fn update_satellite(
    State(repo): State<SatelliteRepository>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateSatelliteDto>,
) -> Result<Json<Satellite>, AppError> {
    let satellite = repo.update_satellite_by_id(id, dto).await?;
    Ok(Json(satellite))
}

#[utoipa::path(
    delete,
    path = "/v1/satellites/{id}",
    params(
        ("id" = Uuid, Path, description = "Satellite unique UUID identifier")
    ),
    responses(
        (status = 204, description = "Satellite deleted successfully"),
        (status = 404, description = "Satellite not found")
    ),
    tag = "Satellites"
)]
pub async fn delete_satellite(
    State(repo): State<SatelliteRepository>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    repo.delete_satellite_by_id(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Triggers automated CelesTrak discovery pipeline sync for a specific satellite group
#[utoipa::path(
    post,
    path = "/v1/pipelines/sync",
    params(PipelineSyncQueryParams),
    responses(
        (status = 200, description = "Pipeline sync completed successfully", body = PipelineSyncResponse)
    ),
    tag = "Pipelines"
)]
pub async fn trigger_pipeline_sync(
    State(repo): State<SatelliteRepository>,
    Query(params): Query<PipelineSyncQueryParams>,
) -> Result<Json<PipelineSyncResponse>, AppError> {
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
        message: format!("Successfully synced {} satellites for group '{}'", synced_count, group_name),
    }))
}

/// Computes the satellite closest to overhead (highest elevation) across all satellites in parallel using **Rayon** with **Tiered Cache**
#[utoipa::path(
    get,
    path = "/v1/satellites/overhead",
    params(OverheadQueryParams),
    responses(
        (status = 200, description = "Overhead satellite computed successfully", body = OverheadResponse),
        (status = 404, description = "No satellite overhead found")
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
                astrodynamics::find_overhead_satellite(&satellites, params.lat, params.lon, alt, time)
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

/// Calculates the next visible pass for a specific satellite above elevation threshold with **Tiered Cache**
#[utoipa::path(
    get,
    path = "/v1/satellites/{id}/next-visible",
    params(
        ("id" = Uuid, Path, description = "Satellite unique UUID identifier"),
        NextVisibleQueryParams
    ),
    responses(
        (status = 200, description = "Next visible pass calculated successfully", body = NextVisiblePassResponse),
        (status = 404, description = "Satellite not found")
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
                    &satellite,
                    params.lat,
                    params.lon,
                    alt,
                    start_time,
                    threshold,
                    1440,
                )
            })
            .await
            .map_err(|e| e.to_string())?;

            pass_res.map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| AppError::InternalServerError(e))?;

    Ok(Json(res))
}
