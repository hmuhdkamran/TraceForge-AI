use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::errors::ApiResult;
use crate::models::test_execution::TestExecution;

pub struct TestExecutionRepo;

impl TestExecutionRepo {
    pub async fn create(
        db: &SqlitePool,
        investigation_id: &str,
        execution_type: &str,
        command_id: &str,
    ) -> ApiResult<TestExecution> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query!(
            "INSERT INTO test_executions (id, investigation_id, execution_type, command_id, status, started_at) VALUES (?, ?, ?, ?, 'running', ?)",
            id,
            investigation_id,
            execution_type,
            command_id,
            now
        )
        .execute(db)
        .await?;

        Self::get_by_id(db, &id).await?.ok_or_else(|| crate::errors::AppError::NotFound("Execution not found".into()))
    }

    pub async fn get_by_id(db: &SqlitePool, id: &str) -> ApiResult<Option<TestExecution>> {
        let row = sqlx::query_as!(
            TestExecution,
            "SELECT id, investigation_id, execution_type, command_id, status, exit_code, started_at, finished_at, duration_ms, stdout_path, stderr_path, summary FROM test_executions WHERE id = ?",
            id
        )
        .fetch_optional(db)
        .await?;
        Ok(row)
    }

    pub async fn complete(
        db: &SqlitePool,
        id: &str,
        exit_code: i64,
        status: &str,
        stdout_path: Option<&str>,
        stderr_path: Option<&str>,
        duration_ms: i64,
        summary: Option<&str>,
    ) -> ApiResult<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query!(
            "UPDATE test_executions SET status = ?, exit_code = ?, finished_at = ?, duration_ms = ?, stdout_path = ?, stderr_path = ?, summary = ? WHERE id = ?",
            status,
            exit_code,
            now,
            duration_ms,
            stdout_path,
            stderr_path,
            summary,
            id
        )
        .execute(db)
        .await?;
        Ok(())
    }

    pub async fn list_for_investigation(db: &SqlitePool, investigation_id: &str, execution_type: Option<&str>) -> ApiResult<Vec<TestExecution>> {
        if let Some(et) = execution_type {
            let rows = sqlx::query_as!(
                TestExecution,
                "SELECT id, investigation_id, execution_type, command_id, status, exit_code, started_at, finished_at, duration_ms, stdout_path, stderr_path, summary FROM test_executions WHERE investigation_id = ? AND execution_type = ? ORDER BY started_at DESC",
                investigation_id,
                et
            )
            .fetch_all(db)
            .await?;
            Ok(rows)
        } else {
            let rows = sqlx::query_as!(
                TestExecution,
                "SELECT id, investigation_id, execution_type, command_id, status, exit_code, started_at, finished_at, duration_ms, stdout_path, stderr_path, summary FROM test_executions WHERE investigation_id = ? ORDER BY started_at DESC",
                investigation_id
            )
            .fetch_all(db)
            .await?;
            Ok(rows)
        }
    }

    pub async fn get_latest(db: &SqlitePool, investigation_id: &str, execution_type: &str) -> ApiResult<Option<TestExecution>> {
        let row = sqlx::query_as!(
            TestExecution,
            "SELECT id, investigation_id, execution_type, command_id, status, exit_code, started_at, finished_at, duration_ms, stdout_path, stderr_path, summary FROM test_executions WHERE investigation_id = ? AND execution_type = ? ORDER BY started_at DESC LIMIT 1",
            investigation_id,
            execution_type
        )
        .fetch_optional(db)
        .await?;
        Ok(row)
    }
}
