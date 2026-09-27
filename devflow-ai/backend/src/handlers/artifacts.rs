use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::json;

use crate::errors::{ApiResult, AppError};
use crate::repositories::{AuditRepo, InvestigationRepo};
use crate::services::artifacts::ArtifactService;
use crate::state::SharedState;

pub async fn sync_artifacts(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    if state.config.is_demo_mode {
        return Err(AppError::DemoModeMutation);
    }

    let investigation = InvestigationRepo::get_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Investigation {} not found", id)))?;

    let result =
        ArtifactService::sync_artifacts(&state.db, &state.config, &id, &investigation.workspace_id)
            .await?;

    // If no errors and findings were imported, advance state cleanly
    if result.errors.is_empty() {
        let current_status = investigation.status.as_str();
        if result.findings_created > 0 {
            if matches!(current_status, "WORKSPACE_READY" | "BASELINE_CAPTURED") {
                let _ =
                    InvestigationRepo::update_status(&state.db, &id, "AWAITING_BOB_INVESTIGATION")
                        .await;
                let _ = AuditRepo::record(
                    &state.db,
                    Some(&id),
                    "status.awaiting_bob_investigation",
                    "system",
                    None,
                    json!({}),
                )
                .await;
            }
            if matches!(
                current_status,
                "WORKSPACE_READY" | "BASELINE_CAPTURED" | "AWAITING_BOB_INVESTIGATION"
            ) {
                let _ = InvestigationRepo::update_status(&state.db, &id, "INVESTIGATION_IMPORTED")
                    .await;
                let _ = AuditRepo::record(
                    &state.db,
                    Some(&id),
                    "status.investigation_imported",
                    "system",
                    None,
                    json!({
                        "artifacts_imported": result.artifacts_imported,
                        "findings_created": result.findings_created,
                    }),
                )
                .await;
            }
            if matches!(
                current_status,
                "WORKSPACE_READY"
                    | "BASELINE_CAPTURED"
                    | "AWAITING_BOB_INVESTIGATION"
                    | "INVESTIGATION_IMPORTED"
            ) {
                let _ = InvestigationRepo::update_status(&state.db, &id, "AWAITING_APPROVAL").await;
                let _ = AuditRepo::record(
                    &state.db,
                    Some(&id),
                    "status.awaiting_approval",
                    "system",
                    None,
                    json!({}),
                )
                .await;
            }
        } else if result.artifacts_imported > 0 {
            if matches!(current_status, "WORKSPACE_READY" | "BASELINE_CAPTURED") {
                let _ =
                    InvestigationRepo::update_status(&state.db, &id, "AWAITING_BOB_INVESTIGATION")
                        .await;
                let _ = AuditRepo::record(
                    &state.db,
                    Some(&id),
                    "status.awaiting_bob_investigation",
                    "system",
                    None,
                    json!({}),
                )
                .await;
            }
            if matches!(
                current_status,
                "WORKSPACE_READY" | "BASELINE_CAPTURED" | "AWAITING_BOB_INVESTIGATION"
            ) {
                let _ = InvestigationRepo::update_status(&state.db, &id, "INVESTIGATION_IMPORTED")
                    .await;
                let _ = AuditRepo::record(
                    &state.db,
                    Some(&id),
                    "status.investigation_imported",
                    "system",
                    None,
                    json!({
                        "artifacts_imported": result.artifacts_imported,
                    }),
                )
                .await;
            }
        }
    }

    Ok(Json(json!({
        "artifacts_imported": result.artifacts_imported,
        "findings_created": result.findings_created,
        "errors": result.errors,
    })))
}

pub async fn list_artifacts(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    InvestigationRepo::get_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Investigation {} not found", id)))?;

    let artifacts = InvestigationRepo::list_artifacts(&state.db, &id).await?;

    Ok(Json(json!({ "artifacts": artifacts })))
}

pub async fn list_findings(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    InvestigationRepo::get_by_id(&state.db, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Investigation {} not found", id)))?;

    let findings = crate::repositories::FindingRepo::list(&state.db, &id).await?;
    let evidence =
        crate::repositories::EvidenceRepo::list_for_investigation(&state.db, &id).await?;

    Ok(Json(json!({
        "findings": findings,
        "evidence": evidence
    })))
}
