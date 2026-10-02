//! Tiered Cache Abstraction Layer (L1 Memory + L2 External Cache)
//!
//! Separates generic cache drivers from cloud-specific providers:
//! - **L1 Memory**: Moka in-memory cache
//! - **Generic L2 Driver**: `redis-rs` protocol client for Redis
//! - **Cloud L2 Provider**: Upstash TLS (`rediss://`) Redis connection

use moka::future::Cache as MokaCache;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use tracing::{debug, warn};

#[derive(Clone)]
pub struct TieredCache {
    l1_cache: MokaCache<String, String>,
    l2_redis: Option<ConnectionManager>,
    default_ttl: Duration,
}

impl TieredCache {
    pub async fn new(redis_url: Option<&str>, default_ttl: Duration) -> Self {
        let l1_cache = MokaCache::builder()
            .max_capacity(10_000)
            .time_to_live(default_ttl)
            .build();

        let effective_redis_url = redis_url
            .map(String::from)
            .or_else(|| std::env::var("REDIS_URL").ok());

        let l2_redis = if let Some(ref url) = effective_redis_url {
            if !url.trim().is_empty() {
                match redis::Client::open(url.as_str()) {
                    Ok(client) => match ConnectionManager::new(client).await {
                        Ok(manager) => {
                            tracing::info!("Connected to L2 Redis Cache at {}", url);
                            Some(manager)
                        }
                        Err(err) => {
                            warn!("Failed to create Redis connection manager: {:?}", err);
                            None
                        }
                    },
                    Err(err) => {
                        warn!("Failed to open Redis client: {:?}", err);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        TieredCache {
            l1_cache,
            l2_redis,
            default_ttl,
        }
    }

    /// Fetches a value from L1 (Memory) or L2 (Redis). If miss, calls fallback function and populates cache.
    pub async fn get_or_insert_with<T, F, Fut>(&self, key: &str, fetch_fn: F) -> Result<T, String>
    where
        T: Serialize + DeserializeOwned + Clone,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, String>>,
    {
        // 1. Check L1 In-Memory Cache
        if let Some(cached_json) = self.l1_cache.get(key).await {
            if let Ok(val) = serde_json::from_str::<T>(&cached_json) {
                debug!("L1 Memory Cache HIT for key: {}", key);
                return Ok(val);
            }
        }

        // 2. Check L2 Redis Cache (if configured)
        if let Some(mut redis_conn) = self.l2_redis.clone() {
            let redis_res: Result<Option<String>, redis::RedisError> = redis_conn.get(key).await;
            if let Ok(Some(cached_json)) = redis_res {
                if let Ok(val) = serde_json::from_str::<T>(&cached_json) {
                    debug!("L2 Redis Cache HIT for key: {}", key);
                    // Populate L1 cache
                    self.l1_cache.insert(key.to_string(), cached_json).await;
                    return Ok(val);
                }
            }
        }

        // 3. Cache MISS -> Fetch from underlying source (DB / Service)
        debug!("Cache MISS for key: {}. Fetching from source...", key);
        let val = fetch_fn().await?;
        let json_str = serde_json::to_string(&val).map_err(|e| e.to_string())?;

        // Populate L1 Memory Cache
        self.l1_cache
            .insert(key.to_string(), json_str.clone())
            .await;

        // Populate L2 Redis Cache (async fire-and-forget)
        if let Some(mut redis_conn) = self.l2_redis.clone() {
            let key_str = key.to_string();
            let ttl_secs = self.default_ttl.as_secs();
            tokio::spawn(async move {
                let _: Result<(), redis::RedisError> =
                    redis_conn.set_ex(key_str, json_str, ttl_secs).await;
            });
        }

        Ok(val)
    }

    /// Invalidates a key across both L1 and L2 caches
    pub async fn invalidate(&self, key: &str) {
        self.l1_cache.invalidate(key).await;

        if let Some(mut redis_conn) = self.l2_redis.clone() {
            let key_str = key.to_string();
            tokio::spawn(async move {
                let _: Result<(), redis::RedisError> = redis_conn.del(key_str).await;
            });
        }
    }
}
