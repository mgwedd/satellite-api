use crate::cache::TieredCache;
use crate::error::AppError;
use crate::models::{CreateSatelliteDto, Satellite, Tle, UpdateSatelliteDto};
use crate::pagination::{
    CheckpointCursor, IdentifiableCheckpoint, PaginatedResponse, PaginationMeta, PaginationQuery,
};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone)]
pub struct SatelliteRepository {
    store: Arc<RwLock<HashMap<Uuid, Satellite>>>,
    pub cache: TieredCache,
}

impl SatelliteRepository {
    pub async fn new(redis_url: Option<&str>) -> Self {
        let cache = TieredCache::new(redis_url, Duration::from_secs(300)).await;
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            cache,
        }
    }

    pub async fn create_satellite(&self, dto: CreateSatelliteDto) -> Result<Satellite, AppError> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let satellite = Satellite {
            id,
            name: dto.name,
            tle: Tle {
                line_one: dto.line_one,
                line_two: dto.line_two,
            },
            created_date: now,
            last_modified_date: now,
        };

        {
            let mut store = self
                .store
                .write()
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;
            store.insert(id, satellite.clone());
        }

        // Invalidate list cache
        self.cache.invalidate("satellites:list").await;

        Ok(satellite)
    }

    pub async fn list_satellites(&self) -> Result<Vec<Satellite>, AppError> {
        let store = self.store.clone();

        self.cache
            .get_or_insert_with("satellites:list", || async move {
                let guard = store
                    .read()
                    .map_err(|e| AppError::InternalServerError(e.to_string()).to_string())?;
                Ok(guard.values().cloned().collect())
            })
            .await
            .map_err(AppError::InternalServerError)
    }

    pub async fn list_satellites_paginated(
        &self,
        query: PaginationQuery,
    ) -> Result<PaginatedResponse<Satellite>, AppError> {
        let limit = query.limit(20, 100);
        let cursor_filter = match &query.cursor {
            Some(c) => Some(
                CheckpointCursor::decode(c)
                    .map_err(|e| AppError::BadRequest(format!("Invalid pagination cursor: {}", e)))?,
            ),
            None => None,
        };

        let mut all_satellites = self.list_satellites().await?;

        // Sort deterministically by (checkpoint_timestamp, checkpoint_id)
        all_satellites.sort_by(|a, b| {
            a.checkpoint_timestamp()
                .cmp(&b.checkpoint_timestamp())
                .then_with(|| a.checkpoint_id().cmp(&b.checkpoint_id()))
        });

        let total_count = all_satellites.len();

        // Apply cursor checkpoint filter
        let filtered: Vec<Satellite> = if let Some(cursor) = cursor_filter {
            all_satellites
                .into_iter()
                .skip_while(|s| {
                    let ts = s.checkpoint_timestamp();
                    let id = s.checkpoint_id();
                    ts < cursor.checkpoint_timestamp
                        || (ts == cursor.checkpoint_timestamp && id <= cursor.last_id)
                })
                .collect()
        } else {
            all_satellites
        };

        let has_more = filtered.len() > limit;
        let page_data: Vec<Satellite> = filtered.into_iter().take(limit).collect();

        let next_cursor = if has_more && !page_data.is_empty() {
            let last_item = page_data.last().unwrap();
            let checkpoint = CheckpointCursor::new(
                last_item.checkpoint_id(),
                last_item.checkpoint_timestamp(),
            );
            checkpoint.encode().ok()
        } else {
            None
        };

        Ok(PaginatedResponse {
            data: page_data,
            pagination: PaginationMeta {
                next_cursor,
                has_more,
                limit,
                total_count,
            },
        })
    }

    pub async fn get_satellite_by_id(&self, id: Uuid) -> Result<Satellite, AppError> {
        let cache_key = format!("satellite:{}", id);
        let store = self.store.clone();

        self.cache
            .get_or_insert_with(&cache_key, || async move {
                let guard = store
                    .read()
                    .map_err(|e| AppError::InternalServerError(e.to_string()).to_string())?;
                guard
                    .get(&id)
                    .cloned()
                    .ok_or_else(|| "Satellite not found".to_string())
            })
            .await
            .map_err(|e| {
                if e == "Satellite not found" {
                    AppError::NotFound
                } else {
                    AppError::InternalServerError(e)
                }
            })
    }

    pub async fn update_satellite_by_id(
        &self,
        id: Uuid,
        dto: UpdateSatelliteDto,
    ) -> Result<Satellite, AppError> {
        let updated_satellite = {
            let mut store = self
                .store
                .write()
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;

            let satellite = store.get_mut(&id).ok_or(AppError::NotFound)?;

            if let Some(name) = dto.name {
                satellite.name = name;
            }
            if let Some(line_one) = dto.line_one {
                satellite.tle.line_one = line_one;
            }
            if let Some(line_two) = dto.line_two {
                satellite.tle.line_two = line_two;
            }
            satellite.last_modified_date = Utc::now();

            satellite.clone()
        };

        // Invalidate both L1 and L2 caches for list and single satellite
        self.cache.invalidate("satellites:list").await;
        self.cache.invalidate(&format!("satellite:{}", id)).await;

        Ok(updated_satellite)
    }

    pub async fn delete_satellite_by_id(&self, id: Uuid) -> Result<Satellite, AppError> {
        let removed_satellite = {
            let mut store = self
                .store
                .write()
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;
            store.remove(&id).ok_or(AppError::NotFound)?
        };

        // Invalidate both L1 and L2 caches
        self.cache.invalidate("satellites:list").await;
        self.cache.invalidate(&format!("satellite:{}", id)).await;

        Ok(removed_satellite)
    }
}
