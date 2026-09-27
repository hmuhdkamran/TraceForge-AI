use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::json;

use crate::errors::{AppError, ApiResult};
use crate::repositories::{InvestigationRepo, TestExecutionRepo};
use crate::services::{investigation::InvestigationService, verification::VerificationService};
use crate::state::SharedState;

pub async fn run_verification(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    if state.config.is_demo_mode {
        return Err(AppError::DemoModeMutation);
    }

    let investigation = InvestigationRepo::get_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Investigation {} not found", id)))?;

    if investigation.approved_at.is_none() {
        return Err(AppError::ApprovalRequired);
    }

    InvestigationService::transition_status(&state.db, &investigation, "VERIFICATION_RUNNING").await?;

    let db = state.db.clone();
    let config = state.config.clone();
    let inv_id = id.clone();
    let inv = investigation.clone();

    tokio::spawn(async move {
        match VerificationService::run_verification(&db, &config, &inv).await {
            Ok(result) => {
                if result.exit_code == 0 {
                    let _ = InvestigationRepo::update_status(&db, &inv_id, "VERIFIED").await;
                } else {
                    let _ = InvestigationRepo::update_status(&db, &inv_id, "VERIFICATION_FAILED").await;
                }
                tracing::info!("Verification complete: {}", result.summary);
            }
            Err(e) => {
                tracing::error!("Verification error: {}", e);
                let _ = InvestigationRepo::update_status(&db, &inv_id, "VERIFICATION_FAILED").await;
            }
        }
    });

    Ok(Json(json!({ "status": "verification started", "investigation_id": id })))
}

pub async fn get_verification(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    use crate::services::workspace::WorkspaceService;

    let investigation = InvestigationRepo::get_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Investigation {} not found", id)))?;

    let baseline = TestExecutionRepo::get_latest(&state.db, &id, "baseline").await?;
    let verification = TestExecutionRepo::get_latest(&state.db, &id, "verification").await?;

    let enrich = |exec: &crate::models::test_execution::TestExecution| -> serde_json::Value {
        let content = exec.stdout_path.as_deref()
            .filter(|p| !p.is_empty())
            .and_then(|p| WorkspaceService::read_artifact_sync(&state.config, &investigation.workspace_id, p))
            .unwrap_or_default();
        json!({
            "id": exec.id,
            "investigation_id": exec.investigation_id,
            "execution_type": exec.execution_type,
            "command_id": exec.command_id,
            "status": exec.status,
            "exit_code": exec.exit_code,
            "started_at": exec.started_at,
            "finished_at": exec.finished_at,
            "duration_ms": exec.duration_ms,
            "summary": exec.summary,
            "stdout": content,
            "stderr": "",
        })
    };

    Ok(Json(json!({
        "baseline": baseline.as_ref().map(enrich),
        "verification": verification.as_ref().map(enrich),
    })))
}

pub async fn get_diff(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let investigation = InvestigationRepo::get_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Investigation {} not found", id)))?;

    // Read implementation summary
    let summary = crate::services::workspace::WorkspaceService::read_artifact(
        &state.config,
        &investigation.workspace_id,
        "bob_artifacts/implementation_summary.md",
    ).await.unwrap_or_else(|_| "Implementation summary not yet available.".into());

    let changed_files: serde_json::Value = match crate::services::workspace::WorkspaceService::read_artifact(
        &state.config,
        &investigation.workspace_id,
        "bob_artifacts/changed_files.json",
    ).await {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => serde_json::Value::Null,
    };

    Ok(Json(json!({
        "implementation_summary": summary,
        "changed_files": changed_files,
    })))
}
