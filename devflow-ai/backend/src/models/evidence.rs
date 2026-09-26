use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Evidence {
    pub id: String,
    pub investigation_id: String,
    pub finding_id: String,
    pub evidence_type: String,
    pub source_file: String,
    pub start_line: Option<i64>,
    pub end_line: Option<i64>,
    pub content_excerpt: Option<String>,
    pub test_id: Option<String>,
    pub description: String,
}
