use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditEvent {
    pub id: String,
    pub investigation_id: Option<String>,
    pub event_type: String,
    pub actor_type: String,
    pub actor_id: Option<String>,
    pub details_json: String,
    pub created_at: String,
}
