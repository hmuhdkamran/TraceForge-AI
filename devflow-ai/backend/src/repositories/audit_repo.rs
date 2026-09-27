use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::errors::ApiResult;
use crate::models::audit::AuditEvent;

pub struct AuditRepo;

impl AuditRepo {
    pub async fn record(
        db: &SqlitePool,
        investigation_id: Option<&str>,
        event_type: &str,
        actor_type: &str,
        actor_id: Option<&str>,
        details: serde_json::Value,
    ) -> ApiResult<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let details_str = details.to_string();

        sqlx::query!(
            "INSERT INTO audit_events (id, investigation_id, event_type, actor_type, actor_id, details_json, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
            id, investigation_id, event_type, actor_type, actor_id, details_str, now
        ).execute(db).await?;
        Ok(())
    }

    pub async fn list_for_investigation(
        db: &SqlitePool,
        investigation_id: &str,
    ) -> ApiResult<Vec<AuditEvent>> {
        let rows = sqlx::query!(
            "SELECT id, investigation_id, event_type, actor_type, actor_id, details_json, created_at FROM audit_events WHERE investigation_id = ? ORDER BY created_at ASC",
            investigation_id
        ).fetch_all(db).await?;
        Ok(rows
            .into_iter()
            .map(|r| AuditEvent {
                id: r.id.unwrap_or_default(),
                investigation_id: r.investigation_id,
                event_type: r.event_type,
                actor_type: r.actor_type,
                actor_id: r.actor_id,
                details_json: r.details_json,
                created_at: r.created_at,
            })
            .collect())
    }
}
