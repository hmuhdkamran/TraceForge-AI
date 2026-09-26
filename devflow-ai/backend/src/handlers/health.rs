use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::state::SharedState;

pub async fn health(State(state): State<SharedState>) -> Json<Value> {
    let db_ok = sqlx::query("SELECT 1").fetch_one(&state.db).await.is_ok();
    Json(json!({
        "status": if db_ok { "ok" } else { "degraded" },
        "service": "contractguard",
        "version": env!("CARGO_PKG_VERSION"),
        "demo_mode": state.config.is_demo_mode,
    }))
}

pub async fn config_handler(State(state): State<SharedState>) -> Json<Value> {
    Json(json!({
        "demo_mode": state.config.is_demo_mode,
        "version": env!("CARGO_PKG_VERSION"),
        "workspace_root": state.config.workspace_root.display().to_string(),
    }))
}
