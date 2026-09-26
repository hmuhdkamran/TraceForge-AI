use axum::{extract::{Path, State}, Json};
use serde_json::json;

use crate::errors::{AppError, ApiResult};
use crate::repositories::InvestigationRepo;
use crate::services::metrics::MetricsService;
use crate::state::SharedState;

pub async fn get_metrics(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let investigation = InvestigationRepo::get_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Investigation {} not found", id)))?;

    let metrics = MetricsService::compute(&state.db, &investigation).await?;
    Ok(Json(json!({ "metrics": metrics })))
}

#[derive(serde::Deserialize)]
pub struct ManualBaselineRequest {
    pub scenario_id: String,
    pub start_time: String,
    pub finish_time: Option<String>,
    pub active_minutes: Option<i64>,
    pub completed_steps: Option<Vec<String>>,
    pub final_test_status: Option<String>,
}

pub async fn record_manual_baseline(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<ManualBaselineRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if state.config.is_demo_mode {
        return Err(AppError::DemoModeMutation);
    }

    let uid = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let steps_json = serde_json::to_string(&req.completed_steps).unwrap_or_default();

    sqlx::query!(
        "INSERT INTO manual_baselines (id, investigation_id, scenario_id, start_time, finish_time, active_minutes, completed_steps, final_test_status, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        uid,
        id,
        req.scenario_id,
        req.start_time,
        req.finish_time,
        req.active_minutes,
        steps_json,
        req.final_test_status,
        now
    )
    .execute(&state.db)
    .await?;

    Ok(Json(json!({ "id": uid, "recorded": true })))
}
