use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::time::timeout;
use sqlx::SqlitePool;

use crate::config::Config;
use crate::errors::{AppError, ApiResult};
use crate::models::investigation::Investigation;
use crate::repositories::{TestExecutionRepo, AuditRepo};

/// Allowed test command definitions. Never accept command IDs from clients.
pub const CMD_RUST_BACKEND_TESTS: &str = "rust_backend_tests";
pub const CMD_FRONTEND_TESTS: &str = "frontend_tests";

pub struct VerificationService;

pub struct ExecutionResult {
    pub execution_id: String,
    pub status: String,
    pub exit_code: i64,
    pub duration_ms: i64,
    pub stdout: String,
    pub stderr: String,
    pub summary: String,
}

impl VerificationService {
    pub async fn run_baseline(
        db: &SqlitePool,
        config: &Config,
        investigation: &Investigation,
        execution_type: &str, // "baseline"
    ) -> ApiResult<ExecutionResult> {
        let workspace = config.workspace_root.join(&investigation.workspace_id);
        let broken_backend = workspace.join("broken").join("backend");

        if !broken_backend.exists() {
            return Err(AppError::Workspace(format!(
                "Backend workspace not found: {}",
                broken_backend.display()
            )));
        }

        Self::run_command(
            db,
            config,
            &investigation.id,
            &investigation.workspace_id,
            execution_type,
            CMD_RUST_BACKEND_TESTS,
            "cargo",
            &["test", "--", "--nocapture"],
            &broken_backend,
        ).await
    }

    pub async fn run_verification(
        db: &SqlitePool,
        config: &Config,
        investigation: &Investigation,
    ) -> ApiResult<ExecutionResult> {
        let workspace = config.workspace_root.join(&investigation.workspace_id);
        let broken_backend = workspace.join("broken").join("backend");

        if !broken_backend.exists() {
            return Err(AppError::Workspace("Backend workspace not found for verification".into()));
        }

        Self::run_command(
            db,
            config,
            &investigation.id,
            &investigation.workspace_id,
            "verification",
            CMD_RUST_BACKEND_TESTS,
            "cargo",
            &["test"],
            &broken_backend,
        ).await
    }

    async fn run_command(
        db: &SqlitePool,
        config: &Config,
        investigation_id: &str,
        workspace_id: &str,
        execution_type: &str,
        command_id: &str,
        program: &str,
        args: &[&str],
        working_dir: &PathBuf,
    ) -> ApiResult<ExecutionResult> {
        // Validate working_dir is under workspace_root
        let canonical_workspace_root = config.workspace_root.canonicalize()
            .unwrap_or_else(|_| config.workspace_root.clone());
        let canonical_working_dir = working_dir.canonicalize()
            .unwrap_or_else(|_| working_dir.clone());
        if !canonical_working_dir.starts_with(&canonical_workspace_root) {
            return Err(AppError::PathTraversal);
        }

        // Create execution record
        let execution = TestExecutionRepo::create(db, investigation_id, execution_type, command_id).await?;

        let start = Instant::now();

        let timeout_duration = Duration::from_secs(config.test_timeout_secs);

        let output_result = timeout(
            timeout_duration,
            Command::new(program)
                .args(args)
                .current_dir(working_dir)
                .output(),
        ).await;

        let duration_ms = start.elapsed().as_millis() as i64;

        match output_result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                // Truncate if needed
                let stdout = Self::truncate_output(&stdout, config.max_output_bytes);
                let stderr = Self::truncate_output(&stderr, config.max_output_bytes);

                let exit_code = output.status.code().unwrap_or(-1) as i64;
                let status = if exit_code == 0 { "passed" } else { "failed" };

                // Save stdout/stderr to workspace files
                let stdout_rel = format!("logs/{}_stdout.txt", execution.id);
                let stderr_rel = format!("logs/{}_stderr.txt", execution.id);

                let log_dir = config.workspace_root.join(workspace_id).join("logs");
                let _ = std::fs::create_dir_all(&log_dir);
                let _ = std::fs::write(log_dir.join(format!("{}_stdout.txt", execution.id)), &stdout);
                let _ = std::fs::write(log_dir.join(format!("{}_stderr.txt", execution.id)), &stderr);

                let summary = Self::build_summary(exit_code, &stdout, &stderr);

                TestExecutionRepo::complete(
                    db,
                    &execution.id,
                    exit_code,
                    status,
                    Some(&stdout_rel),
                    Some(&stderr_rel),
                    duration_ms,
                    Some(&summary),
                ).await?;

                AuditRepo::record(
                    db,
                    Some(investigation_id),
                    &format!("test_execution.{}", status),
                    "system",
                    None,
                    serde_json::json!({
                        "command_id": command_id,
                        "exit_code": exit_code,
                        "duration_ms": duration_ms
                    }),
                ).await?;

                Ok(ExecutionResult {
                    execution_id: execution.id,
                    status: status.to_string(),
                    exit_code,
                    duration_ms,
                    stdout,
                    stderr,
                    summary,
                })
            }
            Ok(Err(e)) => {
                TestExecutionRepo::complete(db, &execution.id, -1, "error", None, None, duration_ms, Some(&e.to_string())).await?;
                Err(AppError::Workspace(format!("Process execution failed: {}", e)))
            }
            Err(_) => {
                TestExecutionRepo::complete(db, &execution.id, -1, "timeout", None, None, duration_ms, Some("Execution timed out")).await?;
                Err(AppError::Workspace("Test execution timed out".into()))
            }
        }
    }

    fn truncate_output(s: &str, max_bytes: usize) -> String {
        if s.len() <= max_bytes {
            s.to_string()
        } else {
            let truncated = &s[..max_bytes];
            format!("{}\n[Output truncated at {} bytes]", truncated, max_bytes)
        }
    }

    fn build_summary(exit_code: i64, stdout: &str, stderr: &str) -> String {
        let combined = format!("{}\n{}", stdout, stderr);

        // Extract test counts from cargo test output
        let mut passed = 0u32;
        let mut failed = 0u32;

        for line in combined.lines() {
            if line.contains("test result:") {
                // e.g. "test result: FAILED. 3 passed; 2 failed;"
                if let Some(p) = Self::extract_count(line, "passed") { passed += p; }
                if let Some(f) = Self::extract_count(line, "failed") { failed += f; }
            }
        }

        if passed > 0 || failed > 0 {
            format!("{} passed, {} failed (exit {})", passed, failed, exit_code)
        } else {
            format!("Exit code: {}", exit_code)
        }
    }

    fn extract_count(line: &str, keyword: &str) -> Option<u32> {
        line.split(keyword)
            .next()
            .and_then(|s| s.split_whitespace().last())
            .and_then(|s| s.parse().ok())
    }
}
