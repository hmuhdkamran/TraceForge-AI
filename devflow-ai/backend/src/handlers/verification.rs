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
    let _ = crate::repositories::AuditRepo::record(
        &state.db,
        Some(&id),
        "status.verification_running",
        "system",
        None,
        json!({}),
    ).await;

    let db = state.db.clone();
    let config = state.config.clone();
    let inv_id = id.clone();
    let inv = investigation.clone();

    tokio::spawn(async move {
        match VerificationService::run_verification(&db, &config, &inv).await {
            Ok(result) => {
                if result.exit_code == 0 {
                    let _ = InvestigationRepo::update_status(&db, &inv_id, "VERIFIED").await;
                    let _ = crate::repositories::AuditRepo::record(
                        &db,
                        Some(&inv_id),
                        "status.verified",
                        "system",
                        None,
                        json!({ "summary": result.summary, "exit_code": result.exit_code }),
                    ).await;
                } else {
                    let _ = InvestigationRepo::update_status(&db, &inv_id, "VERIFICATION_FAILED").await;
                    let _ = crate::repositories::AuditRepo::record(
                        &db,
                        Some(&inv_id),
                        "status.verification_failed",
                        "system",
                        None,
                        json!({ "summary": result.summary, "exit_code": result.exit_code }),
                    ).await;
                }
                tracing::info!("Verification complete: {}", result.summary);
            }
            Err(e) => {
                tracing::error!("Verification error: {}", e);
                let _ = InvestigationRepo::update_status(&db, &inv_id, "VERIFICATION_FAILED").await;
                let _ = crate::repositories::AuditRepo::record(
                    &db,
                    Some(&inv_id),
                    "status.verification_failed",
                    "system",
                    None,
                    json!({ "error": e.to_string() }),
                ).await;
            }
        }
    });

    Ok(Json(json!({ "status": "verification started", "investigation_id": id })))
}

pub async fn get_verification(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    InvestigationRepo::get_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Investigation {} not found", id)))?;

    let baseline = TestExecutionRepo::get_latest(&state.db, &id, "baseline").await?;
    let verification = TestExecutionRepo::get_latest(&state.db, &id, "verification").await?;

    Ok(Json(json!({
        "baseline": baseline,
        "verification": verification,
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

    // Compute diffs for modified files
    let mut file_diffs = Vec::new();
    let mut unified_diff = String::new();

    if let Some(files) = changed_files.get("files").and_then(|f| f.as_array()) {
        for file_obj in files {
            if let Some(rel_path) = file_obj.get("relative_path").and_then(|p| p.as_str()) {
                let clean_rel = rel_path.trim_start_matches('/').trim_start_matches('\\');
                // Original file in sample_project
                let orig_path = state.config.sample_project_root.join(clean_rel);
                // Modified file in isolated workspace
                let modified_path = state.config.workspace_root.join(&investigation.workspace_id).join(clean_rel);

                let orig_content = std::fs::read_to_string(&orig_path).unwrap_or_default();
                let modified_content = std::fs::read_to_string(&modified_path).unwrap_or_default();

                if orig_content != modified_content {
                    let mut diff_str = format!("--- a/{}\n+++ b/{}\n", clean_rel, clean_rel);
                    for d in diff::lines(&orig_content, &modified_content) {
                        match d {
                            diff::Result::Left(l) => diff_str.push_str(&format!("-{}\n", l)),
                            diff::Result::Right(r) => diff_str.push_str(&format!("+{}\n", r)),
                            diff::Result::Both(b, _) => diff_str.push_str(&format!(" {}\n", b)),
                        }
                    }
                    if !unified_diff.is_empty() {
                        unified_diff.push_str("\n");
                    }
                    unified_diff.push_str(&diff_str);

                    file_diffs.push(serde_json::json!({
                        "path": clean_rel,
                        "diff": diff_str
                    }));
                }
            }
        }
    }

    Ok(Json(json!({
        "implementation_summary": summary,
        "changed_files": changed_files,
        "diff": unified_diff,
        "file_diffs": file_diffs
    })))
}
