use crate::config::ServerConfig;
use axum::http::StatusCode;
use dashmap::DashMap;
use deadpool_redis::{Config as DeadpoolConfig, Pool as RedisPool, Runtime};
use moka::future::Cache as MokaCache;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{oneshot, Semaphore};
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

#[derive(Clone)]
pub struct AstreaComputeEngine {
    pub express_pool: Arc<ThreadPool>,
    pub heavy_pool: Arc<ThreadPool>,
    pub express_limit: Arc<Semaphore>,
    pub heavy_limit: Arc<Semaphore>,
    pub user_limits: Arc<DashMap<String, Arc<Semaphore>>>,
    pub flight_tracker: MokaCache<u64, String>,
    pub redis_pool: Option<RedisPool>,
}

impl AstreaComputeEngine {
    pub fn new(config: &ServerConfig) -> Self {
        let express_pool = Arc::new(
            ThreadPoolBuilder::new()
                .num_threads(config.express_cores)
                .thread_name(|i| format!("rayon-express-{}", i))
                .build()
                .expect("Failed to build express Rayon thread pool"),
        );

        let heavy_pool = Arc::new(
            ThreadPoolBuilder::new()
                .num_threads(config.heavy_cores)
                .thread_name(|i| format!("rayon-heavy-{}", i))
                .build()
                .expect("Failed to build heavy Rayon thread pool"),
        );

        let express_limit = Arc::new(Semaphore::new(config.express_cores));
        let heavy_limit = Arc::new(Semaphore::new(config.heavy_cores));
        let user_limits = Arc::new(DashMap::new());

        let flight_tracker = MokaCache::builder()
            .max_capacity(10_000)
            .time_to_live(Duration::from_secs(300))
            .build();

        let redis_pool = if let Some(ref url) = config.redis_url {
            if !url.trim().is_empty() {
                match DeadpoolConfig::from_url(url).create_pool(Some(Runtime::Tokio1)) {
                    Ok(pool) => {
                        tracing::info!(
                            "Initialized Redis L2 connection pool for AstreaComputeEngine"
                        );
                        Some(pool)
                    }
                    Err(err) => {
                        warn!("Failed to create Redis L2 connection pool: {:?}", err);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        Self {
            express_pool,
            heavy_pool,
            express_limit,
            heavy_limit,
            user_limits,
            flight_tracker,
            redis_pool,
        }
    }

    /// O(1) Complexity Gatekeeper
    pub fn estimate_complexity(
        num_satellites: usize,
        duration_days: f64,
        step_seconds: f64,
    ) -> Result<u64, (StatusCode, String)> {
        if step_seconds <= 0.0 {
            return Err((
                StatusCode::BAD_REQUEST,
                "step_seconds must be greater than 0".to_string(),
            ));
        }
        let num_pairs = (num_satellites * (num_satellites.saturating_sub(1))) / 2;
        let total_steps = (duration_days * 86400.0) / step_seconds;
        const COST_PER_STEP_CONSTANT: f64 = 0.001;

        let estimated_ms = (num_pairs as f64 * total_steps * COST_PER_STEP_CONSTANT) as u64;

        if estimated_ms > 300_000 {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                format!(
                    "Compute complexity limit exceeded. Estimated {}ms exceeds maximum allowed limit of 300,000ms",
                    estimated_ms
                ),
            ));
        }

        Ok(estimated_ms)
    }

    /// Executes compute within the L1/L2 Coalescing Swimlane Pipeline
    pub async fn execute_compute<F>(
        &self,
        request_hash: u64,
        estimated_ms: u64,
        user_id: String,
        user_role: String,
        compute_closure: F,
    ) -> Result<String, (StatusCode, String)>
    where
        F: FnOnce(CancellationToken) -> Result<String, String> + Send + 'static,
    {
        let cache_key = format!("astrea:compute:{}", request_hash);

        let redis_pool = self.redis_pool.clone();
        let user_role_clone = user_role.clone();
        let user_id_clone = user_id.clone();
        let user_limits = self.user_limits.clone();
        let express_pool = self.express_pool.clone();
        let heavy_pool = self.heavy_pool.clone();
        let express_limit = self.express_limit.clone();
        let heavy_limit = self.heavy_limit.clone();

        let result = self
            .flight_tracker
            .try_get_with(request_hash, async move {
                // 1. L2 Distributed Cache (Cluster-Global) via persistent Connection Pool
                if let Some(ref pool) = redis_pool {
                    if let Ok(mut conn) = pool.get().await {
                        if let Ok(cached) = redis::cmd("GET")
                            .arg(&cache_key)
                            .query_async::<_, String>(&mut *conn)
                            .await
                        {
                            debug!("L2 Redis HIT for compute key: {}", cache_key);
                            return Ok(cached);
                        }
                    }
                }

                // 2. User Identity Quota (Role == "admin" bypasses user semaphore)
                let _user_permit = if user_role_clone != "admin" {
                    let sem = user_limits
                        .entry(user_id_clone)
                        .or_insert_with(|| Arc::new(Semaphore::new(3)))
                        .clone();
                    Some(
                        sem.acquire_owned()
                            .await
                            .map_err(|_| "User quota exceeded".to_string())?,
                    )
                } else {
                    None
                };

                // 3. Swimlane Routing (Rayon Pool selection based on estimated_ms)
                let (pool, global_sem) = if estimated_ms <= 500 {
                    (&express_pool, &express_limit)
                } else {
                    (&heavy_pool, &heavy_limit)
                };

                let _global_permit = global_sem
                    .acquire()
                    .await
                    .map_err(|_| "Compute capacity full".to_string())?;

                let (tx, rx) = oneshot::channel();
                let cancel_token = CancellationToken::new();
                let token_clone = cancel_token.clone();

                // 4. Rayon Handoff
                pool.spawn(move || {
                    let _ = tx.send(compute_closure(token_clone));
                });

                // 5. Timeboxing & Ghost Task Prevention
                let _drop_guard = cancel_token.clone().drop_guard();
                let data = tokio::time::timeout(Duration::from_secs(30), rx)
                    .await
                    .map_err(|_| "Execution timed out".to_string())?
                    .map_err(|_| "Worker panicked".to_string())??;

                // 6. Populate L2 Cache via persistent Connection Pool
                if let Some(ref pool) = redis_pool {
                    if let Ok(mut conn) = pool.get().await {
                        let _ = redis::cmd("SETEX")
                            .arg(&cache_key)
                            .arg(300)
                            .arg(&data)
                            .query_async::<_, ()>(&mut *conn)
                            .await;
                    }
                }

                Ok(data)
            })
            .await;

        result.map_err(|e: std::sync::Arc<String>| {
            (StatusCode::INTERNAL_SERVER_ERROR, e.as_ref().clone())
        })
    }
}
