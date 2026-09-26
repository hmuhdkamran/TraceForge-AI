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

        sqlx::query!(
            "INSERT INTO evidence (id, investigation_id, finding_id, evidence_type, source_file, start_line, end_line, content_excerpt, test_id, description) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            id, investigation_id, finding_id, evidence_type, source_file, start_line, end_line, content_excerpt, test_id, description
        ).execute(db).await?;

        let row = sqlx::query!(
            "SELECT id, investigation_id, finding_id, evidence_type, source_file, start_line, end_line, content_excerpt, test_id, description FROM evidence WHERE id = ?",
            id
        ).fetch_one(db).await?;

        Ok(Evidence {
            id: row.id.unwrap_or_default(),
            investigation_id: row.investigation_id,
            finding_id: row.finding_id,
            evidence_type: row.evidence_type,
            source_file: row.source_file,
            start_line: row.start_line,
            end_line: row.end_line,
            content_excerpt: row.content_excerpt,
            test_id: row.test_id,
            description: row.description,
        })
    }

    pub async fn list_for_investigation(db: &SqlitePool, investigation_id: &str) -> ApiResult<Vec<Evidence>> {
        let rows = sqlx::query!(
            "SELECT id, investigation_id, finding_id, evidence_type, source_file, start_line, end_line, content_excerpt, test_id, description FROM evidence WHERE investigation_id = ? ORDER BY rowid ASC",
            investigation_id
        ).fetch_all(db).await?;
        Ok(rows.into_iter().map(|r| Evidence {
            id: r.id.unwrap_or_default(),
            investigation_id: r.investigation_id,
            finding_id: r.finding_id,
            evidence_type: r.evidence_type,
            source_file: r.source_file,
            start_line: r.start_line,
            end_line: r.end_line,
            content_excerpt: r.content_excerpt,
            test_id: r.test_id,
            description: r.description,
        }).collect())
    }

    pub async fn delete_for_investigation(db: &SqlitePool, investigation_id: &str) -> ApiResult<()> {
        sqlx::query!("DELETE FROM evidence WHERE investigation_id = ?", investigation_id)
            .execute(db).await?;
        Ok(())
    }
}
