use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Finding {
    pub id: String,
    pub investigation_id: String,
    pub finding_type: String,
    pub title: String,
    pub description: String,
    pub expected_behavior: String,
    pub observed_behavior: String,
    pub confidence: String,
    pub verification_status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFindingRequest {
    pub finding_type: String,
    pub title: String,
    pub description: String,
    pub expected_behavior: String,
    pub observed_behavior: String,
    pub confidence: String,
}
