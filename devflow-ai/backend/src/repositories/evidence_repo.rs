use sqlx::SqlitePool;
use uuid::Uuid;

use crate::errors::ApiResult;
use crate::models::evidence::Evidence;

pub struct EvidenceRepo;

impl EvidenceRepo {
    pub async fn create(
        db: &SqlitePool,
        investigation_id: &str,
        finding_id: &str,
        evidence_type: &str,
        source_file: &str,
        start_line: Option<i64>,
        end_line: Option<i64>,
        content_excerpt: Option<&str>,
        test_id: Option<&str>,
        description: &str,
    ) -> ApiResult<Evidence> {
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO evidence (id, investigation_id, finding_id, evidence_type, source_file, start_line, end_line, content_excerpt, test_id, description) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(investigation_id)
        .bind(finding_id)
        .bind(evidence_type)
        .bind(source_file)
        .bind(start_line)
        .bind(end_line)
        .bind(content_excerpt)
        .bind(test_id)
        .bind(description)
        .execute(db)
        .await?;

        let row = sqlx::query_as::<_, Evidence>(
            "SELECT id, investigation_id, finding_id, evidence_type, source_file, start_line, end_line, content_excerpt, test_id, description FROM evidence WHERE id = ?"
        )
        .bind(&id)
        .fetch_one(db)
        .await?;
        Ok(row)
    }

    pub async fn list_for_investigation(db: &SqlitePool, investigation_id: &str) -> ApiResult<Vec<Evidence>> {
        let rows = sqlx::query_as::<_, Evidence>(
            "SELECT id, investigation_id, finding_id, evidence_type, source_file, start_line, end_line, content_excerpt, test_id, description FROM evidence WHERE investigation_id = ? ORDER BY rowid ASC"
        )
        .bind(investigation_id)
        .fetch_all(db)
        .await?;
        Ok(rows)
    }

    pub async fn delete_for_investigation(db: &SqlitePool, investigation_id: &str) -> ApiResult<()> {
        sqlx::query("DELETE FROM evidence WHERE investigation_id = ?")
            .bind(investigation_id)
            .execute(db)
            .await?;
        Ok(())
    }
}
