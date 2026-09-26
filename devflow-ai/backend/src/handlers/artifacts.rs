use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::json;

use crate::errors::{AppError, ApiResult};
use crate::repositories::InvestigationRepo;
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

    let result = ArtifactService::sync_artifacts(
        &state.db,
        &state.config,
        &id,
        &investigation.workspace_id,
    ).await?;

    // If findings were imported, advance state if needed
    if result.findings_created > 0 && matches!(
        investigation.status.as_str(),
        "AWAITING_BOB_INVESTIGATION" | "INVESTIGATION_IMPORTED"
    ) {
        let _ = InvestigationRepo::update_status(&state.db, &id, "INVESTIGATION_IMPORTED").await;
        let _ = InvestigationRepo::update_status(&state.db, &id, "AWAITING_APPROVAL").await;
    } else if result.artifacts_imported > 0 && investigation.status == "AWAITING_BOB_INVESTIGATION" {
        let _ = InvestigationRepo::update_status(&state.db, &id, "INVESTIGATION_IMPORTED").await;
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
    let evidence = crate::repositories::EvidenceRepo::list_for_investigation(&state.db, &id).await?;

    Ok(Json(json!({
        "findings": findings,
        "evidence": evidence
    })))
}
