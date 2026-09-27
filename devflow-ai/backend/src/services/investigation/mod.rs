use serde_json::json;
use sqlx::SqlitePool;

use crate::config::Config;
use crate::errors::{ApiResult, AppError};
use crate::models::investigation::{
    CreateInvestigationRequest, Investigation, InvestigationStatus,
};
use crate::repositories::{AuditRepo, InvestigationRepo};
use crate::services::workspace::WorkspaceService;

pub struct InvestigationService;

impl InvestigationService {
    pub async fn create(
        db: &SqlitePool,
        config: &Config,
        req: CreateInvestigationRequest,
    ) -> ApiResult<Investigation> {
        if req.title.trim().is_empty() {
            return Err(AppError::BadRequest("Title is required".into()));
        }
        if req.description.trim().is_empty() {
            return Err(AppError::BadRequest("Description is required".into()));
        }

        let investigation = InvestigationRepo::create(
            db,
            &req.title,
            &req.description,
            &req.project_id,
            &req.scenario_id,
        )
        .await?;

        AuditRepo::record(
            db,
            Some(&investigation.id),
            "investigation.created",
            "developer",
            None,
            json!({
                "title": req.title,
                "project_id": req.project_id,
                "scenario_id": req.scenario_id
            }),
        )
        .await?;

        // Create workspace
        match WorkspaceService::create_workspace(
            config,
            &investigation.id,
            &investigation.workspace_id,
        )
        .await
        {
            Ok(_) => {
                InvestigationRepo::update_status(db, &investigation.id, "WORKSPACE_READY").await?;
                AuditRepo::record(
                    db,
                    Some(&investigation.id),
                    "workspace.created",
                    "system",
                    None,
                    json!({}),
                )
                .await?;
            }
            Err(e) => {
                tracing::error!("Failed to create workspace: {}", e);
                InvestigationRepo::set_failure(
                    db,
                    &investigation.id,
                    &format!("Workspace creation failed: {}", e),
                )
                .await?;
            }
        }

        InvestigationRepo::get_by_id(db, &investigation.id)
            .await?
            .ok_or_else(|| AppError::NotFound("Investigation not found after creation".into()))
    }

    pub async fn transition_status(
        db: &SqlitePool,
        investigation: &Investigation,
        next_status: &str,
    ) -> ApiResult<()> {
        let current = Self::parse_status(&investigation.status)?;
        let next = Self::parse_status(next_status)?;

        if !current.can_transition_to(&next) {
            return Err(AppError::InvalidTransition(format!(
                "Cannot transition from {} to {}",
                investigation.status, next_status
            )));
        }

        InvestigationRepo::update_status(db, &investigation.id, next_status).await
    }

    fn parse_status(s: &str) -> ApiResult<InvestigationStatus> {
        match s {
            "CREATED" => Ok(InvestigationStatus::Created),
            "WORKSPACE_READY" => Ok(InvestigationStatus::WorkspaceReady),
            "BASELINE_RUNNING" => Ok(InvestigationStatus::BaselineRunning),
            "BASELINE_CAPTURED" => Ok(InvestigationStatus::BaselineCaptured),
            "AWAITING_BOB_INVESTIGATION" => Ok(InvestigationStatus::AwaitingBobInvestigation),
            "INVESTIGATION_IMPORTED" => Ok(InvestigationStatus::InvestigationImported),
            "AWAITING_APPROVAL" => Ok(InvestigationStatus::AwaitingApproval),
            "APPROVED" => Ok(InvestigationStatus::Approved),
            "IMPLEMENTATION_IN_PROGRESS" => Ok(InvestigationStatus::ImplementationInProgress),
            "READY_FOR_VERIFICATION" => Ok(InvestigationStatus::ReadyForVerification),
            "VERIFICATION_RUNNING" => Ok(InvestigationStatus::VerificationRunning),
            "VERIFIED" => Ok(InvestigationStatus::Verified),
            "VERIFICATION_FAILED" => Ok(InvestigationStatus::VerificationFailed),
            "COMPLETED" => Ok(InvestigationStatus::Completed),
            "FAILED" => Ok(InvestigationStatus::Failed),
            other => Err(AppError::BadRequest(format!("Unknown status: {}", other))),
        }
    }
}
