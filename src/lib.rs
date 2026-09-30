pub mod cache;
pub mod config;
pub mod error;
pub mod handlers;
pub mod models;
pub mod pagination;
pub mod repository;
pub mod services;

use axum::{
    routing::{get, post},
    Router,
};
use repository::SatelliteRepository;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub fn create_router(repo: SatelliteRepository) -> Router {
    let api_routes = Router::new()
        .route(
            "/satellites",
            post(handlers::create_satellite).get(handlers::list_satellites),
        )
        .route(
            "/satellites/:id",
            get(handlers::get_satellite)
                .patch(handlers::update_satellite)
                .delete(handlers::delete_satellite),
        )
        .route("/satellites/overhead", get(handlers::get_overhead))
        .route(
            "/satellites/:id/next-visible",
            get(handlers::get_next_visible),
        )
        .route("/pipelines/sync", post(handlers::trigger_pipeline_sync))
        .with_state(repo);

    Router::new()
        .nest("/v1", api_routes)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
