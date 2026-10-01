pub mod auth;
pub mod cache;
pub mod config;
pub mod error;
pub mod handlers;
pub mod models;
pub mod pagination;
pub mod repository;
pub mod services;

use axum::{
    response::{Html, Redirect},
    routing::{get, post},
    Router,
};
use repository::SatelliteRepository;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use utoipa::{
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::login_handler,
        handlers::satellite_handler::create_satellite,
        handlers::satellite_handler::list_satellites,
        handlers::satellite_handler::get_satellite,
        handlers::satellite_handler::update_satellite,
        handlers::satellite_handler::delete_satellite,
        handlers::satellite_handler::trigger_pipeline_sync,
        handlers::satellite_handler::get_overhead,
        handlers::satellite_handler::get_next_visible,
        handlers::satellite_handler::get_ground_track,
    ),
    components(
        schemas(
            auth::UserRole,
            auth::Claims,
            auth::LoginRequest,
            auth::AuthResponse,
            models::Satellite,
            models::Tle,
            models::CreateSatelliteDto,
            models::UpdateSatelliteDto,
            models::OverheadResponse,
            models::NextVisiblePassResponse,
            models::GroundTrackResponse,
            models::GroundTrackPoint,
            models::GeoJsonFeature,
            models::GeoJsonGeometry,
            handlers::satellite_handler::PipelineSyncResponse,
            pagination::PaginationMeta,
            pagination::PaginatedResponseSatellite,
            error::ErrorResponse,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Authentication", description = "JWT Token issuance endpoints"),
        (name = "Satellites", description = "Satellite management endpoints"),
        (name = "Astrodynamics", description = "Orbital calculations and pass predictions"),
        (name = "Pipelines", description = "CelesTrak automated discovery pipelines")
    )
)]
pub struct ApiDoc;

pub fn create_router(repo: SatelliteRepository) -> Router {
    let api_routes = Router::new()
        .route("/auth/login", post(auth::login_handler))
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
        .route(
            "/satellites/:id/groundtrack",
            get(handlers::get_ground_track),
        )
        .route("/pipelines/sync", post(handlers::trigger_pipeline_sync))
        .with_state(repo);

    const REDOC_HTML: &str = r#"<!DOCTYPE html>
<html>
  <head>
    <title>Satellite API - Interactive OpenAPI Documentation</title>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap" rel="stylesheet">
    <style>body { margin: 0; padding: 0; font-family: 'Inter', sans-serif; }</style>
  </head>
  <body>
    <redoc spec-url='/api-docs/openapi.json' expand-responses="200,201"></redoc>
    <script src="https://cdn.jsdelivr.net/npm/redoc@latest/bundles/redoc.standalone.js"></script>
  </body>
</html>"#;

    Router::new()
        .route("/", get(|| async { Redirect::temporary("/swagger-ui/") }))
        .route("/docs", get(|| async { Html(REDOC_HTML) }))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/v1", api_routes)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
