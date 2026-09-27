use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "TEXT", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvestigationStatus {
    Created,
    WorkspaceReady,
    BaselineRunning,
    BaselineCaptured,
    AwaitingBobInvestigation,
    InvestigationImported,
    AwaitingApproval,
    Approved,
    ImplementationInProgress,
    ReadyForVerification,
    VerificationRunning,
    Verified,
    VerificationFailed,
    Completed,
    Failed,
}

impl InvestigationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "CREATED",
            Self::WorkspaceReady => "WORKSPACE_READY",
            Self::BaselineRunning => "BASELINE_RUNNING",
            Self::BaselineCaptured => "BASELINE_CAPTURED",
            Self::AwaitingBobInvestigation => "AWAITING_BOB_INVESTIGATION",
            Self::InvestigationImported => "INVESTIGATION_IMPORTED",
            Self::AwaitingApproval => "AWAITING_APPROVAL",
            Self::Approved => "APPROVED",
            Self::ImplementationInProgress => "IMPLEMENTATION_IN_PROGRESS",
            Self::ReadyForVerification => "READY_FOR_VERIFICATION",
            Self::VerificationRunning => "VERIFICATION_RUNNING",
            Self::Verified => "VERIFIED",
            Self::VerificationFailed => "VERIFICATION_FAILED",
            Self::Completed => "COMPLETED",
            Self::Failed => "FAILED",
        }
    }

    pub fn can_transition_to(&self, next: &InvestigationStatus) -> bool {
        match (self, next) {
            (Self::Created, Self::WorkspaceReady) => true,
            (Self::Created, Self::Failed) => true,
            (Self::WorkspaceReady, Self::BaselineRunning) => true,
            (Self::WorkspaceReady, Self::AwaitingBobInvestigation) => true,
            (Self::WorkspaceReady, Self::Failed) => true,
            (Self::BaselineRunning, Self::BaselineCaptured) => true,
            (Self::BaselineRunning, Self::Failed) => true,
            (Self::BaselineCaptured, Self::AwaitingBobInvestigation) => true,
            (Self::BaselineCaptured, Self::BaselineRunning) => true, // re-run baseline
            (Self::AwaitingBobInvestigation, Self::BaselineRunning) => true, // re-run baseline
            (Self::AwaitingBobInvestigation, Self::InvestigationImported) => true,
            (Self::InvestigationImported, Self::AwaitingApproval) => true,
            (Self::InvestigationImported, Self::InvestigationImported) => true, // re-import
            (Self::AwaitingApproval, Self::Approved) => true,
            (Self::AwaitingApproval, Self::AwaitingApproval) => true, // re-import with changes
            (Self::Approved, Self::ImplementationInProgress) => true,
            (Self::ImplementationInProgress, Self::ReadyForVerification) => true,
            (Self::ImplementationInProgress, Self::Failed) => true,
            (Self::ReadyForVerification, Self::VerificationRunning) => true,
            (Self::VerificationRunning, Self::Verified) => true,
            (Self::VerificationRunning, Self::VerificationFailed) => true,
            (Self::Verified, Self::Completed) => true,
            (Self::VerificationFailed, Self::ImplementationInProgress) => true, // retry
            (_, Self::Failed) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Investigation {
    pub id: String,
    pub title: String,
    pub description: String,
    pub project_id: String,
    pub scenario_id: String,
    pub status: String,
    pub workspace_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub approved_at: Option<String>,
    pub completed_at: Option<String>,
    pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvestigationRequest {
    pub title: String,
    pub description: String,
    pub project_id: String,
    pub scenario_id: String,
    pub expected_behavior: Option<String>,
    pub observed_behavior: Option<String>,
    pub reproduction_steps: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct InvestigationArtifact {
    pub id: String,
    pub investigation_id: String,
    pub artifact_type: String,
    pub schema_version: i64,
    pub relative_path: String,
    pub content_sha256: String,
    pub created_at: String,
    pub imported_at: String,
    pub validation_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Approval {
    pub id: String,
    pub investigation_id: String,
    pub plan_hash: String,
    pub decision: String,
    pub approved_at: String,
    pub approver_id: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub decision: String, // "approved" | "rejected"
    pub comment: Option<String>,
    pub approver_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: String,
    pub bug_report: String,
}
