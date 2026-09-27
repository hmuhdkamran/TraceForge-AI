use axum::{
    extract::{Path, State},
    response::Html,
    Json,
};
use serde_json::json;

use crate::errors::{AppError, ApiResult};
use crate::repositories::InvestigationRepo;
use crate::services::reporting::ReportingService;
use crate::state::SharedState;

pub async fn get_report(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let investigation = InvestigationRepo::get_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Investigation {} not found", id)))?;

    let report = ReportingService::generate(&state.db, &state.config, &investigation).await?;
    Ok(Json(json!({ "report": report })))
}

pub async fn get_report_html(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> ApiResult<Html<String>> {
    let investigation = InvestigationRepo::get_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Investigation {} not found", id)))?;

    let report = ReportingService::generate(&state.db, &state.config, &investigation).await?;
    let html = ReportingService::render_html(&report);

    // Mark as completed if verified
    if investigation.status == "VERIFIED" {
        let _ = InvestigationRepo::set_completed_at(&state.db, &id).await;
    }

    Ok(Html(html))
}

pub async fn get_report_json(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let investigation = InvestigationRepo::get_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Investigation {} not found", id)))?;

    let report = ReportingService::generate(&state.db, &state.config, &investigation).await?;
    Ok(Json(serde_json::to_value(&report)?))
}
