pub mod config;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod security;
pub mod services;
pub mod state;

use axum::Router;
use sqlx::SqlitePool;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::state::AppState;

pub async fn app(config: Config) -> anyhow::Result<Router> {
    // Initialize DB
    let pool = SqlitePool::connect(&config.database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = Arc::new(AppState::new(pool, config.clone()));

    let cors = if config.is_demo_mode {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any)
    } else {
        let origins: Vec<_> = config
            .allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any)
    };

    let router = Router::new()
        .merge(routes::all_routes(state))
        .layer(cors)
        .layer(RequestBodyLimitLayer::new(config.max_body_bytes))
        .layer(TraceLayer::new_for_http());

    Ok(router)
}
