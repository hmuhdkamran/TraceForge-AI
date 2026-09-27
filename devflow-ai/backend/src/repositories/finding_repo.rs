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

        sqlx::query(
            "INSERT INTO findings (id, investigation_id, finding_type, title, description, expected_behavior, observed_behavior, confidence, verification_status, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'unverified', ?)"
        )
        .bind(&id)
        .bind(investigation_id)
        .bind(finding_type)
        .bind(title)
        .bind(description)
        .bind(expected_behavior)
        .bind(observed_behavior)
        .bind(confidence)
        .bind(&now)
        .execute(db)
        .await?;

        let row = sqlx::query_as::<_, Finding>(
            "SELECT id, investigation_id, finding_type, title, description, expected_behavior, observed_behavior, confidence, verification_status, created_at FROM findings WHERE id = ?"
        )
        .bind(&id)
        .fetch_one(db)
        .await?;
        Ok(row)
    }

    pub async fn list(db: &SqlitePool, investigation_id: &str) -> ApiResult<Vec<Finding>> {
        let rows = sqlx::query_as::<_, Finding>(
            "SELECT id, investigation_id, finding_type, title, description, expected_behavior, observed_behavior, confidence, verification_status, created_at FROM findings WHERE investigation_id = ? ORDER BY created_at ASC"
        )
        .bind(investigation_id)
        .fetch_all(db)
        .await?;
        Ok(rows)
    }

    pub async fn delete_for_investigation(db: &SqlitePool, investigation_id: &str) -> ApiResult<()> {
        sqlx::query("DELETE FROM findings WHERE investigation_id = ?")
            .bind(investigation_id)
            .execute(db)
            .await?;
        Ok(())
    }
}
