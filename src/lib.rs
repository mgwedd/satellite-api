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
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::satellite_handler::create_satellite,
        handlers::satellite_handler::list_satellites,
        handlers::satellite_handler::get_satellite,
        handlers::satellite_handler::update_satellite,
        handlers::satellite_handler::delete_satellite,
        handlers::satellite_handler::trigger_pipeline_sync,
        handlers::satellite_handler::get_overhead,
        handlers::satellite_handler::get_next_visible,
    ),
    components(
        schemas(
            models::Satellite,
            models::Tle,
            models::CreateSatelliteDto,
            models::UpdateSatelliteDto,
            models::OverheadResponse,
            models::NextVisiblePassResponse,
            handlers::satellite_handler::PipelineSyncResponse,
            pagination::PaginationMeta,
            pagination::PaginatedResponseSatellite,
            error::ErrorResponse,
        )
    ),
    tags(
        (name = "Satellites", description = "Satellite management endpoints"),
        (name = "Astrodynamics", description = "Orbital calculations and pass predictions"),
        (name = "Pipelines", description = "CelesTrak automated discovery pipelines")
    )
)]
pub struct ApiDoc;

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
        .route("/astrodynamics/overhead", get(handlers::get_overhead))
        .route(
            "/satellites/:id/next-visible",
            get(handlers::get_next_visible),
        )
        .route("/pipelines/sync", post(handlers::trigger_pipeline_sync))
        .with_state(repo);

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/v1", api_routes)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
