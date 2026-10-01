use crate::cache::TieredCache;
use crate::error::AppError;
use crate::models::{CreateSatelliteDto, Satellite, Tle, UpdateSatelliteDto};
use crate::pagination::{
    CheckpointCursor, IdentifiableCheckpoint, PaginatedResponse, PaginationMeta, PaginationQuery,
};
use crate::repository::data_provider::{
    DataProvider, MemoryDataProvider, SupabasePostgresDataProvider,
};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone)]
pub struct SatelliteRepository {
    pub provider: Arc<dyn DataProvider>,
    pub cache: TieredCache,
}

impl SatelliteRepository {
    pub async fn new(redis_url: Option<&str>) -> Self {
        let cache = TieredCache::new(redis_url, Duration::from_secs(300)).await;

        let provider: Arc<dyn DataProvider> = if let Ok(db_url) = std::env::var("DATABASE_URL") {
            if !db_url.trim().is_empty() {
                if let Ok(pg) = SupabasePostgresDataProvider::connect(&db_url).await {
                    Arc::new(pg)
                } else {
                    Arc::new(MemoryDataProvider::new())
                }
            } else {
                Arc::new(MemoryDataProvider::new())
            }
        } else {
            Arc::new(MemoryDataProvider::new())
        };

        Self { provider, cache }
    }

    pub fn with_provider(provider: Arc<dyn DataProvider>, cache: TieredCache) -> Self {
        Self { provider, cache }
    }

    pub async fn create_satellite(&self, dto: CreateSatelliteDto) -> Result<Satellite, AppError> {
        let sat = self.provider.create_satellite(dto).await?;
        self.cache.invalidate("satellites:list").await;
        Ok(sat)
    }

    pub async fn list_satellites(&self) -> Result<Vec<Satellite>, AppError> {
        let provider = self.provider.clone();
        self.cache
            .get_or_insert_with("satellites:list", || async move {
                provider.list_satellites().await.map_err(|e| e.to_string())
            })
            .await
            .map_err(AppError::InternalServerError)
    }

    pub async fn list_satellites_paginated(
        &self,
        query: PaginationQuery,
    ) -> Result<PaginatedResponse<Satellite>, AppError> {
        let limit = query.limit(20, 100);
        let cursor_filter =
            match &query.cursor {
                Some(c) => Some(CheckpointCursor::decode(c).map_err(|e| {
                    AppError::BadRequest(format!("Invalid pagination cursor: {}", e))
                })?),
                None => None,
            };

        let mut all_satellites = self.list_satellites().await?;
        all_satellites.sort_by(|a, b| {
            a.checkpoint_timestamp()
                .cmp(&b.checkpoint_timestamp())
                .then_with(|| a.checkpoint_id().cmp(&b.checkpoint_id()))
        });

        let total_count = all_satellites.len();

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
            let checkpoint =
                CheckpointCursor::new(last_item.checkpoint_id(), last_item.checkpoint_timestamp());
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
        let provider = self.provider.clone();

        self.cache
            .get_or_insert_with(&cache_key, || async move {
                provider.get_satellite(id).await.map_err(|e| e.to_string())
            })
            .await
            .map_err(|e| {
                if e.contains("NotFound") || e.contains("not found") {
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
        let updated = self.provider.update_satellite(id, dto).await?;
        self.cache.invalidate("satellites:list").await;
        self.cache.invalidate(&format!("satellite:{}", id)).await;
        Ok(updated)
    }

    pub async fn delete_satellite_by_id(&self, id: Uuid) -> Result<Satellite, AppError> {
        let sat = self.get_satellite_by_id(id).await?;
        self.provider.delete_satellite(id).await?;
        self.cache.invalidate("satellites:list").await;
        self.cache.invalidate(&format!("satellite:{}", id)).await;
        Ok(sat)
    }

    pub async fn list_tle_history(&self, satellite_id: Uuid) -> Result<Vec<Tle>, AppError> {
        self.provider.list_tle_history(satellite_id).await
    }

    pub async fn add_tle_history(&self, satellite_id: Uuid, tle: Tle) -> Result<(), AppError> {
        self.provider.add_tle_history(satellite_id, tle).await
    }
}
