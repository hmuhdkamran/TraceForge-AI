use axum::{
    extract::{Path, Query, State},
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::errors::{AppError, ApiResult};
use crate::models::investigation::CreateInvestigationRequest;
use crate::repositories::{InvestigationRepo, AuditRepo};
use crate::services::{
    investigation::InvestigationService,
    prompts::PromptService,
};
use crate::state::SharedState;

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn create_investigation(
    State(state): State<SharedState>,
    Json(req): Json<CreateInvestigationRequest>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    if state.config.is_demo_mode {
        return Err(AppError::DemoModeMutation);
    }

    let investigation = InvestigationService::create(&state.db, &state.config, req).await?;

    Ok((StatusCode::CREATED, Json(json!({ "investigation": investigation }))))
}

pub async fn list_investigations(
    State(state): State<SharedState>,
    Query(params): Query<PaginationParams>,
) -> ApiResult<Json<Value>> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let investigations = InvestigationRepo::list(&state.db, limit, offset).await?;
    let total = InvestigationRepo::count(&state.db).await?;

    Ok(Json(json!({
        "investigations": investigations,
        "total": total,
        "limit": limit,
        "offset": offset
    })))
}

pub async fn get_investigation(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let investigation = InvestigationRepo::get_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Investigation {} not found", id)))?;

    Ok(Json(json!({ "investigation": investigation })))
}

pub async fn get_investigation_events(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    InvestigationRepo::get_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Investigation {} not found", id)))?;

    let events = AuditRepo::list_for_investigation(&state.db, &id).await?;
    Ok(Json(json!({ "events": events })))
}

pub async fn run_baseline(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    if state.config.is_demo_mode {
        return Err(AppError::DemoModeMutation);
    }

    let investigation = InvestigationRepo::get_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Investigation {} not found", id)))?;

    // Update status
    InvestigationService::transition_status(&state.db, &investigation, "BASELINE_RUNNING").await?;

    let db = state.db.clone();
    let config = state.config.clone();
    let inv_id = id.clone();

    // Run in background
    tokio::spawn(async move {
        match crate::services::verification::VerificationService::run_baseline(
            &db, &config, &investigation, "baseline"
        ).await {
            Ok(result) => {
                let _ = InvestigationRepo::update_status(&db, &inv_id, "BASELINE_CAPTURED").await;
                let _ = InvestigationRepo::update_status(&db, &inv_id, "AWAITING_BOB_INVESTIGATION").await;
                tracing::info!("Baseline complete: {}", result.summary);
            }
            Err(e) => {
                tracing::error!("Baseline failed: {}", e);
                let _ = InvestigationRepo::update_status(&db, &inv_id, "BASELINE_CAPTURED").await;
                let _ = InvestigationRepo::update_status(&db, &inv_id, "AWAITING_BOB_INVESTIGATION").await;
            }
        }
    });

    Ok(Json(json!({ "status": "baseline started", "investigation_id": id })))
}

pub async fn get_baseline(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    use crate::repositories::TestExecutionRepo;

    InvestigationRepo::get_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Investigation {} not found", id)))?;

    let executions = TestExecutionRepo::list_for_investigation(
        &state.db, &id, Some("baseline")
    ).await?;

    Ok(Json(json!({ "baseline_executions": executions })))
}

pub async fn get_bob_investigation_prompt(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let investigation = InvestigationRepo::get_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Investigation {} not found", id)))?;

    let prompt = PromptService::generate_investigation_prompt(&investigation, &state.config);
    let workspace_path = state.config.workspace_root.join(&investigation.workspace_id).display().to_string();

    Ok(Json(json!({
        "prompt": prompt,
        "workspace_path": workspace_path,
        "required_artifacts": [
            "bob_artifacts/diagnosis.json",
            "bob_artifacts/investigation_summary.md",
            "bob_artifacts/fix_plan.md"
        ]
    })))
}

pub async fn get_bob_implementation_prompt(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let investigation = InvestigationRepo::get_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Investigation {} not found", id)))?;

    if investigation.approved_at.is_none() {
        return Err(AppError::ApprovalRequired);
    }

    let approval = InvestigationRepo::get_approval(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::ApprovalRequired)?;

    // Read fix_plan
    let fix_plan = crate::services::workspace::WorkspaceService::read_artifact(
        &state.config,
        &investigation.workspace_id,
        "bob_artifacts/fix_plan.md",
    ).await.unwrap_or_else(|_| "Fix plan not yet available.".into());

    let prompt = PromptService::generate_implementation_prompt(
        &investigation,
        &state.config,
        &approval.plan_hash,
        &fix_plan,
    );

    Ok(Json(json!({
        "prompt": prompt,
        "plan_hash": approval.plan_hash,
    })))
}
