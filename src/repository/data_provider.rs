use crate::error::AppError;
use crate::models::{CreateSatelliteDto, Satellite, Tle, UpdateSatelliteDto};
use axum::async_trait;
use chrono::Utc;
use sqlx::Row;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[async_trait]
pub trait DataProvider: Send + Sync {
    async fn create_satellite(&self, dto: CreateSatelliteDto) -> Result<Satellite, AppError>;
    async fn get_satellite(&self, id: Uuid) -> Result<Satellite, AppError>;
    async fn list_satellites(&self) -> Result<Vec<Satellite>, AppError>;
    async fn update_satellite(
        &self,
        id: Uuid,
        dto: UpdateSatelliteDto,
    ) -> Result<Satellite, AppError>;
    async fn delete_satellite(&self, id: Uuid) -> Result<(), AppError>;
    async fn list_tle_history(&self, satellite_id: Uuid) -> Result<Vec<Tle>, AppError>;
    async fn add_tle_history(&self, satellite_id: Uuid, tle: Tle) -> Result<(), AppError>;
}

/// In-memory DataProvider fallback for offline testing and fast development.
pub struct MemoryDataProvider {
    store: Arc<RwLock<HashMap<Uuid, Satellite>>>,
    tle_history: Arc<RwLock<HashMap<Uuid, Vec<Tle>>>>,
}

impl MemoryDataProvider {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            tle_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryDataProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DataProvider for MemoryDataProvider {
    async fn create_satellite(&self, dto: CreateSatelliteDto) -> Result<Satellite, AppError> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let tle = Tle {
            line_one: dto.line_one,
            line_two: dto.line_two,
        };
        let satellite = Satellite {
            id,
            name: dto.name,
            tle: tle.clone(),
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
        {
            let mut history = self
                .tle_history
                .write()
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;
            history.entry(id).or_default().push(tle);
        }

        Ok(satellite)
    }

    async fn get_satellite(&self, id: Uuid) -> Result<Satellite, AppError> {
        let store = self
            .store
            .read()
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        store.get(&id).cloned().ok_or(AppError::NotFound)
    }

    async fn list_satellites(&self) -> Result<Vec<Satellite>, AppError> {
        let store = self
            .store
            .read()
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        Ok(store.values().cloned().collect())
    }

    async fn update_satellite(
        &self,
        id: Uuid,
        dto: UpdateSatelliteDto,
    ) -> Result<Satellite, AppError> {
        let mut store = self
            .store
            .write()
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let satellite = store.get_mut(&id).ok_or(AppError::NotFound)?;

        if let Some(name) = dto.name {
            satellite.name = name;
        }

        let mut tle_updated = false;
        if let Some(l1) = dto.line_one {
            satellite.tle.line_one = l1;
            tle_updated = true;
        }
        if let Some(l2) = dto.line_two {
            satellite.tle.line_two = l2;
            tle_updated = true;
        }

        satellite.last_modified_date = Utc::now();
        let updated = satellite.clone();

        if tle_updated {
            let mut history = self
                .tle_history
                .write()
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;
            history.entry(id).or_default().push(updated.tle.clone());
        }

        Ok(updated)
    }

    async fn delete_satellite(&self, id: Uuid) -> Result<(), AppError> {
        let mut store = self
            .store
            .write()
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        if store.remove(&id).is_none() {
            return Err(AppError::NotFound);
        }
        let mut history = self
            .tle_history
            .write()
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        history.remove(&id);
        Ok(())
    }

    async fn list_tle_history(&self, satellite_id: Uuid) -> Result<Vec<Tle>, AppError> {
        let history_opt = {
            let history = self
                .tle_history
                .read()
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;
            history.get(&satellite_id).cloned()
        };
        if let Some(list) = history_opt {
            Ok(list)
        } else {
            let sat = self.get_satellite(satellite_id).await?;
            Ok(vec![sat.tle])
        }
    }

    async fn add_tle_history(&self, satellite_id: Uuid, tle: Tle) -> Result<(), AppError> {
        let mut history = self
            .tle_history
            .write()
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        history.entry(satellite_id).or_default().push(tle);
        Ok(())
    }
}

/// Native PostgreSQL DataProvider utilizing SQLx connection pool and Row-Level Security (RLS).
pub struct PostgresDataProvider {
    pool: sqlx::PgPool,
}

pub type SupabasePostgresDataProvider = PostgresDataProvider;

impl PostgresDataProvider {
    pub async fn connect(db_url: &str) -> Result<Self, AppError> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .connect(db_url)
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Database connection failed: {}", e))
            })?;

        // Automatically run initial schema setup / migrations
        let _ = sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'viewer',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE TABLE IF NOT EXISTS satellites (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                name TEXT NOT NULL,
                line_one TEXT NOT NULL,
                line_two TEXT NOT NULL,
                owner_id TEXT,
                created_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                last_modified_date TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE TABLE IF NOT EXISTS tle_history (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                satellite_id UUID NOT NULL REFERENCES satellites(id) ON DELETE CASCADE,
                line_one TEXT NOT NULL,
                line_two TEXT NOT NULL,
                epoch TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            ALTER TABLE satellites ENABLE ROW LEVEL SECURITY;
            ALTER TABLE tle_history ENABLE ROW LEVEL SECURITY;
            ALTER TABLE users ENABLE ROW LEVEL SECURITY;
            "#,
        )
        .execute(&pool)
        .await;

        Ok(Self { pool })
    }

    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DataProvider for PostgresDataProvider {
    async fn create_satellite(&self, dto: CreateSatelliteDto) -> Result<Satellite, AppError> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO satellites (id, name, line_one, line_two, created_date, last_modified_date) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(id)
        .bind(&dto.name)
        .bind(&dto.line_one)
        .bind(&dto.line_two)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to insert satellite: {}", e)))?;

        sqlx::query(
            "INSERT INTO tle_history (satellite_id, line_one, line_two, epoch) VALUES ($1, $2, $3, $4)"
        )
        .bind(id)
        .bind(&dto.line_one)
        .bind(&dto.line_two)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to insert TLE history: {}", e)))?;

        Ok(Satellite {
            id,
            name: dto.name,
            tle: Tle {
                line_one: dto.line_one,
                line_two: dto.line_two,
            },
            created_date: now,
            last_modified_date: now,
        })
    }

    async fn get_satellite(&self, id: Uuid) -> Result<Satellite, AppError> {
        let row = sqlx::query(
            "SELECT id, name, line_one, line_two, created_date, last_modified_date FROM satellites WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        if let Some(r) = row {
            Ok(Satellite {
                id: r
                    .try_get("id")
                    .map_err(|e| AppError::InternalServerError(e.to_string()))?,
                name: r
                    .try_get("name")
                    .map_err(|e| AppError::InternalServerError(e.to_string()))?,
                tle: Tle {
                    line_one: r
                        .try_get("line_one")
                        .map_err(|e| AppError::InternalServerError(e.to_string()))?,
                    line_two: r
                        .try_get("line_two")
                        .map_err(|e| AppError::InternalServerError(e.to_string()))?,
                },
                created_date: r
                    .try_get("created_date")
                    .map_err(|e| AppError::InternalServerError(e.to_string()))?,
                last_modified_date: r
                    .try_get("last_modified_date")
                    .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            })
        } else {
            Err(AppError::NotFound)
        }
    }

    async fn list_satellites(&self) -> Result<Vec<Satellite>, AppError> {
        let rows = sqlx::query(
            "SELECT id, name, line_one, line_two, created_date, last_modified_date FROM satellites",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let mut list = Vec::with_capacity(rows.len());
        for r in rows {
            list.push(Satellite {
                id: r
                    .try_get("id")
                    .map_err(|e| AppError::InternalServerError(e.to_string()))?,
                name: r
                    .try_get("name")
                    .map_err(|e| AppError::InternalServerError(e.to_string()))?,
                tle: Tle {
                    line_one: r
                        .try_get("line_one")
                        .map_err(|e| AppError::InternalServerError(e.to_string()))?,
                    line_two: r
                        .try_get("line_two")
                        .map_err(|e| AppError::InternalServerError(e.to_string()))?,
                },
                created_date: r
                    .try_get("created_date")
                    .map_err(|e| AppError::InternalServerError(e.to_string()))?,
                last_modified_date: r
                    .try_get("last_modified_date")
                    .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            });
        }
        Ok(list)
    }

    async fn update_satellite(
        &self,
        id: Uuid,
        dto: UpdateSatelliteDto,
    ) -> Result<Satellite, AppError> {
        let current = self.get_satellite(id).await?;
        let name = dto.name.unwrap_or(current.name);
        let line_one = dto.line_one.unwrap_or(current.tle.line_one);
        let line_two = dto.line_two.unwrap_or(current.tle.line_two);
        let now = Utc::now();

        sqlx::query(
            "UPDATE satellites SET name = $1, line_one = $2, line_two = $3, last_modified_date = $4 WHERE id = $5"
        )
        .bind(&name)
        .bind(&line_one)
        .bind(&line_two)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        sqlx::query(
            "INSERT INTO tle_history (satellite_id, line_one, line_two, epoch) VALUES ($1, $2, $3, $4)"
        )
        .bind(id)
        .bind(&line_one)
        .bind(&line_two)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(Satellite {
            id,
            name,
            tle: Tle { line_one, line_two },
            created_date: current.created_date,
            last_modified_date: now,
        })
    }

    async fn delete_satellite(&self, id: Uuid) -> Result<(), AppError> {
        let res = sqlx::query("DELETE FROM satellites WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        if res.rows_affected() == 0 {
            Err(AppError::NotFound)
        } else {
            Ok(())
        }
    }

    async fn list_tle_history(&self, satellite_id: Uuid) -> Result<Vec<Tle>, AppError> {
        let rows = sqlx::query(
            "SELECT line_one, line_two FROM tle_history WHERE satellite_id = $1 ORDER BY epoch ASC",
        )
        .bind(satellite_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        if rows.is_empty() {
            let sat = self.get_satellite(satellite_id).await?;
            Ok(vec![sat.tle])
        } else {
            let mut list = Vec::with_capacity(rows.len());
            for r in rows {
                list.push(Tle {
                    line_one: r
                        .try_get("line_one")
                        .map_err(|e| AppError::InternalServerError(e.to_string()))?,
                    line_two: r
                        .try_get("line_two")
                        .map_err(|e| AppError::InternalServerError(e.to_string()))?,
                });
            }
            Ok(list)
        }
    }

    async fn add_tle_history(&self, satellite_id: Uuid, tle: Tle) -> Result<(), AppError> {
        let now = Utc::now();
        sqlx::query(
            "INSERT INTO tle_history (satellite_id, line_one, line_two, epoch) VALUES ($1, $2, $3, $4)"
        )
        .bind(satellite_id)
        .bind(&tle.line_one)
        .bind(&tle.line_two)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        Ok(())
    }
}

/// Supabase Cloud DataProvider utilizing the official Supabase PostgREST client library (`postgrest`).
pub struct SupabaseDataProvider {
    client: postgrest::Postgrest,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct SupabaseSatelliteRow {
    id: Uuid,
    name: String,
    line_one: String,
    line_two: String,
    created_date: chrono::DateTime<Utc>,
    last_modified_date: chrono::DateTime<Utc>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct SupabaseTleRow {
    line_one: String,
    line_two: String,
}

impl SupabaseDataProvider {
    pub fn new(supabase_url: String, apikey: String) -> Self {
        let endpoint = format!("{}/rest/v1", supabase_url.trim_end_matches('/'));
        let client = postgrest::Postgrest::new(endpoint)
            .insert_header("apikey", &apikey)
            .insert_header("Authorization", format!("Bearer {}", apikey));
        Self { client }
    }
}

#[async_trait]
impl DataProvider for SupabaseDataProvider {
    async fn create_satellite(&self, dto: CreateSatelliteDto) -> Result<Satellite, AppError> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let body = serde_json::json!({
            "id": id,
            "name": dto.name,
            "line_one": dto.line_one,
            "line_two": dto.line_two,
            "created_date": now,
            "last_modified_date": now
        });

        let resp = self
            .client
            .from("satellites")
            .insert(body.to_string())
            .execute()
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Supabase create request failed: {}", e))
            })?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(AppError::InternalServerError(format!(
                "Supabase create satellite failed: {}",
                err_text
            )));
        }

        let history_body = serde_json::json!({
            "satellite_id": id,
            "line_one": dto.line_one,
            "line_two": dto.line_two,
            "epoch": now
        });
        let _ = self
            .client
            .from("tle_history")
            .insert(history_body.to_string())
            .execute()
            .await;

        Ok(Satellite {
            id,
            name: dto.name,
            tle: Tle {
                line_one: dto.line_one,
                line_two: dto.line_two,
            },
            created_date: now,
            last_modified_date: now,
        })
    }

    async fn get_satellite(&self, id: Uuid) -> Result<Satellite, AppError> {
        let resp = self
            .client
            .from("satellites")
            .select("*")
            .eq("id", id.to_string())
            .execute()
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Supabase request failed: {}", e))
            })?;

        if !resp.status().is_success() {
            return Err(AppError::NotFound);
        }

        let body_text = resp
            .text()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let rows: Vec<SupabaseSatelliteRow> = serde_json::from_str(&body_text).map_err(|e| {
            AppError::InternalServerError(format!("Invalid Supabase payload: {}", e))
        })?;

        let r = rows.into_iter().next().ok_or(AppError::NotFound)?;
        Ok(Satellite {
            id: r.id,
            name: r.name,
            tle: Tle {
                line_one: r.line_one,
                line_two: r.line_two,
            },
            created_date: r.created_date,
            last_modified_date: r.last_modified_date,
        })
    }

    async fn list_satellites(&self) -> Result<Vec<Satellite>, AppError> {
        let resp = self
            .client
            .from("satellites")
            .select("*")
            .execute()
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Supabase request failed: {}", e))
            })?;

        if !resp.status().is_success() {
            return Ok(vec![]);
        }

        let body_text = resp
            .text()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let rows: Vec<SupabaseSatelliteRow> = serde_json::from_str(&body_text).map_err(|e| {
            AppError::InternalServerError(format!("Invalid Supabase payload: {}", e))
        })?;

        Ok(rows
            .into_iter()
            .map(|r| Satellite {
                id: r.id,
                name: r.name,
                tle: Tle {
                    line_one: r.line_one,
                    line_two: r.line_two,
                },
                created_date: r.created_date,
                last_modified_date: r.last_modified_date,
            })
            .collect())
    }

    async fn update_satellite(
        &self,
        id: Uuid,
        dto: UpdateSatelliteDto,
    ) -> Result<Satellite, AppError> {
        let current = self.get_satellite(id).await?;
        let name = dto.name.unwrap_or(current.name);
        let line_one = dto.line_one.unwrap_or(current.tle.line_one);
        let line_two = dto.line_two.unwrap_or(current.tle.line_two);
        let now = Utc::now();

        let body = serde_json::json!({
            "name": name,
            "line_one": line_one,
            "line_two": line_two,
            "last_modified_date": now
        });

        let resp = self
            .client
            .from("satellites")
            .eq("id", id.to_string())
            .update(body.to_string())
            .execute()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Supabase update failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(AppError::NotFound);
        }

        let history_body = serde_json::json!({
            "satellite_id": id,
            "line_one": line_one,
            "line_two": line_two,
            "epoch": now
        });
        let _ = self
            .client
            .from("tle_history")
            .insert(history_body.to_string())
            .execute()
            .await;

        Ok(Satellite {
            id,
            name,
            tle: Tle { line_one, line_two },
            created_date: current.created_date,
            last_modified_date: now,
        })
    }

    async fn delete_satellite(&self, id: Uuid) -> Result<(), AppError> {
        let resp = self
            .client
            .from("satellites")
            .eq("id", id.to_string())
            .delete()
            .execute()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Supabase delete failed: {}", e)))?;

        if !resp.status().is_success() {
            Err(AppError::NotFound)
        } else {
            Ok(())
        }
    }

    async fn list_tle_history(&self, satellite_id: Uuid) -> Result<Vec<Tle>, AppError> {
        let resp = self
            .client
            .from("tle_history")
            .select("line_one,line_two")
            .eq("satellite_id", satellite_id.to_string())
            .order("epoch.asc")
            .execute()
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Supabase request failed: {}", e))
            })?;

        if !resp.status().is_success() {
            let sat = self.get_satellite(satellite_id).await?;
            return Ok(vec![sat.tle]);
        }

        let body_text = resp.text().await.unwrap_or_default();
        let rows: Vec<SupabaseTleRow> = serde_json::from_str(&body_text).unwrap_or_default();

        if rows.is_empty() {
            let sat = self.get_satellite(satellite_id).await?;
            Ok(vec![sat.tle])
        } else {
            Ok(rows
                .into_iter()
                .map(|r| Tle {
                    line_one: r.line_one,
                    line_two: r.line_two,
                })
                .collect())
        }
    }

    async fn add_tle_history(&self, satellite_id: Uuid, tle: Tle) -> Result<(), AppError> {
        let now = Utc::now();
        let body = serde_json::json!({
            "satellite_id": satellite_id,
            "line_one": tle.line_one,
            "line_two": tle.line_two,
            "epoch": now
        });

        let resp = self
            .client
            .from("tle_history")
            .insert(body.to_string())
            .execute()
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Supabase insert TLE failed: {}", e))
            })?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            Err(AppError::InternalServerError(format!(
                "Supabase insert TLE history failed: {}",
                err_text
            )))
        } else {
            Ok(())
        }
    }
}
