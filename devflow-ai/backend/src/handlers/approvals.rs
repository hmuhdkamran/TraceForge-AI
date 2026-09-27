use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::json;

use crate::errors::{AppError, ApiResult};
use crate::models::investigation::ApprovalRequest;
use crate::repositories::InvestigationRepo;
use crate::services::artifacts::ArtifactService;
use crate::state::SharedState;

pub async fn get_plan(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let investigation = InvestigationRepo::get_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Investigation {} not found", id)))?;

    let fix_plan = crate::services::workspace::WorkspaceService::read_artifact(
        &state.config,
        &investigation.workspace_id,
        "bob_artifacts/fix_plan.md",
    ).await.unwrap_or_default();

    let plan_hash = ArtifactService::sha256(&fix_plan);
    let approval = InvestigationRepo::get_approval(&state.db, &id).await?;

    Ok(Json(json!({
        "fix_plan": fix_plan,
        "plan_hash": plan_hash,
        "approval": approval,
        "investigation_status": investigation.status,
    })))
}

pub async fn create_approval(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<ApprovalRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    if state.config.is_demo_mode {
        return Err(AppError::DemoModeMutation);
    }

    let investigation = InvestigationRepo::get_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Investigation {} not found", id)))?;

    if !matches!(investigation.status.as_str(), "AWAITING_APPROVAL" | "INVESTIGATION_IMPORTED") {
        return Err(AppError::InvalidTransition(format!(
            "Cannot approve in status: {}", investigation.status
        )));
    }

    // Compute plan hash
    let fix_plan = crate::services::workspace::WorkspaceService::read_artifact(
        &state.config,
        &investigation.workspace_id,
        "bob_artifacts/fix_plan.md",
    ).await.unwrap_or_default();

    if fix_plan.is_empty() {
        return Err(AppError::BadRequest("fix_plan.md is empty or missing — cannot approve without a plan".into()));
    }

    let plan_hash = ArtifactService::sha256(&fix_plan);

    let decision = req.decision.to_lowercase();
    if !matches!(decision.as_str(), "approved" | "rejected") {
        return Err(AppError::BadRequest("Decision must be 'approved' or 'rejected'".into()));
    }

    let approval = InvestigationRepo::create_approval(
        &state.db,
        &id,
        &plan_hash,
        &decision,
        req.approver_id.as_deref(),
        req.comment.as_deref(),
    ).await?;

    if decision == "approved" {
        InvestigationRepo::set_approved_at(&state.db, &id).await?;
        let _ = crate::repositories::AuditRepo::record(
            &state.db,
            Some(&id),
            "status.approved",
            "developer",
            req.approver_id.as_deref(),
            json!({ "plan_hash": plan_hash }),
        ).await;
    }

    crate::repositories::AuditRepo::record(
        &state.db,
        Some(&id),
        &format!("approval.{}", decision),
        "developer",
        req.approver_id.as_deref(),
        json!({ "plan_hash": plan_hash, "comment": req.comment }),
    ).await?;

    Ok(Json(json!({ "approval": approval })))
}
