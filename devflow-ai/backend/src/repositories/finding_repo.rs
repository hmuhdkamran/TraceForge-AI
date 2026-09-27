use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::errors::ApiResult;
use crate::models::finding::Finding;

pub struct FindingRepo;

impl FindingRepo {
    pub async fn create(
        db: &SqlitePool,
        investigation_id: &str,
        finding_type: &str,
        title: &str,
        description: &str,
        expected_behavior: &str,
        observed_behavior: &str,
        confidence: &str,
    ) -> ApiResult<Finding> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query!(
            "INSERT INTO findings (id, investigation_id, finding_type, title, description, expected_behavior, observed_behavior, confidence, verification_status, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'unverified', ?)",
            id, investigation_id, finding_type, title, description, expected_behavior, observed_behavior, confidence, now
        ).execute(db).await?;

        let row = sqlx::query!(
            "SELECT id, investigation_id, finding_type, title, description, expected_behavior, observed_behavior, confidence, verification_status, created_at FROM findings WHERE id = ?",
            id
        ).fetch_one(db).await?;

        Ok(Finding {
            id: row.id.unwrap_or_default(),
            investigation_id: row.investigation_id,
            finding_type: row.finding_type,
            title: row.title,
            description: row.description,
            expected_behavior: row.expected_behavior,
            observed_behavior: row.observed_behavior,
            confidence: row.confidence,
            verification_status: row.verification_status,
            created_at: row.created_at,
        })
    }

    pub async fn list(db: &SqlitePool, investigation_id: &str) -> ApiResult<Vec<Finding>> {
        let rows = sqlx::query!(
            "SELECT id, investigation_id, finding_type, title, description, expected_behavior, observed_behavior, confidence, verification_status, created_at FROM findings WHERE investigation_id = ? ORDER BY created_at ASC",
            investigation_id
        ).fetch_all(db).await?;
        Ok(rows
            .into_iter()
            .map(|r| Finding {
                id: r.id.unwrap_or_default(),
                investigation_id: r.investigation_id,
                finding_type: r.finding_type,
                title: r.title,
                description: r.description,
                expected_behavior: r.expected_behavior,
                observed_behavior: r.observed_behavior,
                confidence: r.confidence,
                verification_status: r.verification_status,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn delete_for_investigation(
        db: &SqlitePool,
        investigation_id: &str,
    ) -> ApiResult<()> {
        sqlx::query!(
            "DELETE FROM findings WHERE investigation_id = ?",
            investigation_id
        )
        .execute(db)
        .await?;
        Ok(())
    }
}
