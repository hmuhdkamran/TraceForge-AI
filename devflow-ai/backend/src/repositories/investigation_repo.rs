use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::errors::{AppError, ApiResult};
use crate::models::investigation::{Investigation, InvestigationArtifact, Approval, ApprovalRequest};

pub struct InvestigationRepo;

impl InvestigationRepo {
    pub async fn create(
        db: &SqlitePool,
        title: &str,
        description: &str,
        project_id: &str,
        scenario_id: &str,
    ) -> ApiResult<Investigation> {
        let id = Uuid::new_v4().to_string();
        let workspace_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query!(
            r#"INSERT INTO investigations
                (id, title, description, project_id, scenario_id, status, workspace_id, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, 'CREATED', ?, ?, ?)"#,
            id,
            title,
            description,
            project_id,
            scenario_id,
            workspace_id,
            now,
            now,
        )
        .execute(db)
        .await?;

        Self::get_by_id(db, &id).await?.ok_or_else(|| AppError::NotFound("Investigation not found after create".into()))
    }

    pub async fn get_by_id(db: &SqlitePool, id: &str) -> ApiResult<Option<Investigation>> {
        let row = sqlx::query_as!(
            Investigation,
            "SELECT id, title, description, project_id, scenario_id, status, workspace_id, created_at, updated_at, approved_at, completed_at, failure_reason FROM investigations WHERE id = ?",
            id
        )
        .fetch_optional(db)
        .await?;
        Ok(row)
    }

    pub async fn list(db: &SqlitePool, limit: i64, offset: i64) -> ApiResult<Vec<Investigation>> {
        let rows = sqlx::query_as!(
            Investigation,
            "SELECT id, title, description, project_id, scenario_id, status, workspace_id, created_at, updated_at, approved_at, completed_at, failure_reason FROM investigations ORDER BY created_at DESC LIMIT ? OFFSET ?",
            limit,
            offset
        )
        .fetch_all(db)
        .await?;
        Ok(rows)
    }

    pub async fn count(db: &SqlitePool) -> ApiResult<i64> {
        let row = sqlx::query!("SELECT COUNT(*) as cnt FROM investigations")
            .fetch_one(db)
            .await?;
        Ok(row.cnt)
    }

    pub async fn update_status(db: &SqlitePool, id: &str, status: &str) -> ApiResult<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query!(
            "UPDATE investigations SET status = ?, updated_at = ? WHERE id = ?",
            status,
            now,
            id
        )
        .execute(db)
        .await?;
        Ok(())
    }

    pub async fn set_approved_at(db: &SqlitePool, id: &str) -> ApiResult<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query!(
            "UPDATE investigations SET approved_at = ?, updated_at = ?, status = 'APPROVED' WHERE id = ?",
            now,
            now,
            id
        )
        .execute(db)
        .await?;
        Ok(())
    }

    pub async fn set_completed_at(db: &SqlitePool, id: &str) -> ApiResult<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query!(
            "UPDATE investigations SET completed_at = ?, updated_at = ?, status = 'COMPLETED' WHERE id = ?",
            now,
            now,
            id
        )
        .execute(db)
        .await?;
        Ok(())
    }

    pub async fn set_failure(db: &SqlitePool, id: &str, reason: &str) -> ApiResult<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query!(
            "UPDATE investigations SET status = 'FAILED', failure_reason = ?, updated_at = ? WHERE id = ?",
            reason,
            now,
            id
        )
        .execute(db)
        .await?;
        Ok(())
    }

    // Artifacts
    pub async fn upsert_artifact(
        db: &SqlitePool,
        investigation_id: &str,
        artifact_type: &str,
        relative_path: &str,
        content_sha256: &str,
        validation_status: &str,
    ) -> ApiResult<InvestigationArtifact> {
        let now = Utc::now().to_rfc3339();

        // Check if artifact with same path already exists
        let existing = sqlx::query_as!(
            InvestigationArtifact,
            "SELECT id, investigation_id, artifact_type, schema_version, relative_path, content_sha256, created_at, imported_at, validation_status FROM investigation_artifacts WHERE investigation_id = ? AND relative_path = ?",
            investigation_id,
            relative_path
        )
        .fetch_optional(db)
        .await?;

        if let Some(existing) = existing {
            // Update if sha changed
            if existing.content_sha256 != content_sha256 {
                sqlx::query!(
                    "UPDATE investigation_artifacts SET content_sha256 = ?, imported_at = ?, validation_status = ? WHERE id = ?",
                    content_sha256,
                    now,
                    validation_status,
                    existing.id
                )
                .execute(db)
                .await?;
            }
            return Self::get_artifact_by_id(db, &existing.id).await?
                .ok_or_else(|| AppError::NotFound("Artifact not found after update".into()));
        }

        let id = Uuid::new_v4().to_string();
        sqlx::query!(
            "INSERT INTO investigation_artifacts (id, investigation_id, artifact_type, schema_version, relative_path, content_sha256, created_at, imported_at, validation_status) VALUES (?, ?, ?, 1, ?, ?, ?, ?, ?)",
            id,
            investigation_id,
            artifact_type,
            relative_path,
            content_sha256,
            now,
            now,
            validation_status
        )
        .execute(db)
        .await?;

        Self::get_artifact_by_id(db, &id).await?
            .ok_or_else(|| AppError::NotFound("Artifact not found after insert".into()))
    }

    pub async fn get_artifact_by_id(db: &SqlitePool, id: &str) -> ApiResult<Option<InvestigationArtifact>> {
        let row = sqlx::query_as!(
            InvestigationArtifact,
            "SELECT id, investigation_id, artifact_type, schema_version, relative_path, content_sha256, created_at, imported_at, validation_status FROM investigation_artifacts WHERE id = ?",
            id
        )
        .fetch_optional(db)
        .await?;
        Ok(row)
    }

    pub async fn list_artifacts(db: &SqlitePool, investigation_id: &str) -> ApiResult<Vec<InvestigationArtifact>> {
        let rows = sqlx::query_as!(
            InvestigationArtifact,
            "SELECT id, investigation_id, artifact_type, schema_version, relative_path, content_sha256, created_at, imported_at, validation_status FROM investigation_artifacts WHERE investigation_id = ? ORDER BY imported_at DESC",
            investigation_id
        )
        .fetch_all(db)
        .await?;
        Ok(rows)
    }

    // Approvals
    pub async fn create_approval(
        db: &SqlitePool,
        investigation_id: &str,
        plan_hash: &str,
        decision: &str,
        approver_id: Option<&str>,
        comment: Option<&str>,
    ) -> ApiResult<Approval> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query!(
            "INSERT INTO approvals (id, investigation_id, plan_hash, decision, approved_at, approver_id, comment) VALUES (?, ?, ?, ?, ?, ?, ?)",
            id,
            investigation_id,
            plan_hash,
            decision,
            now,
            approver_id,
            comment
        )
        .execute(db)
        .await?;

        Self::get_approval(db, investigation_id).await?
            .ok_or_else(|| AppError::NotFound("Approval not found after create".into()))
    }

    pub async fn get_approval(db: &SqlitePool, investigation_id: &str) -> ApiResult<Option<Approval>> {
        let row = sqlx::query_as!(
            Approval,
            "SELECT id, investigation_id, plan_hash, decision, approved_at, approver_id, comment FROM approvals WHERE investigation_id = ? ORDER BY approved_at DESC LIMIT 1",
            investigation_id
        )
        .fetch_optional(db)
        .await?;
        Ok(row)
    }
}
