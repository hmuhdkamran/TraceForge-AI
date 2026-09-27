use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TestExecution {
    pub id: String,
    pub investigation_id: String,
    pub execution_type: String,
    pub command_id: String,
    pub status: String,
    pub exit_code: Option<i64>,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub duration_ms: Option<i64>,
    pub stdout_path: Option<String>,
    pub stderr_path: Option<String>,
    pub summary: Option<String>,
}
