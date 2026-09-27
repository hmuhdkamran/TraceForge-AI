use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::time::timeout;
use sqlx::SqlitePool;

use crate::config::Config;
use crate::errors::{AppError, ApiResult};
use crate::models::investigation::Investigation;
use crate::repositories::{TestExecutionRepo, AuditRepo};
use crate::security::path_guard::PathGuard;

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
            execution_type,
            CMD_RUST_BACKEND_TESTS,
            "cargo",
            &["test"],
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
        execution_type: &str,
        command_id: &str,
        program: &str,
        args: &[&str],
        working_dir: &PathBuf,
    ) -> ApiResult<ExecutionResult> {
        // Validate working_dir is under workspace_root using canonical absolute paths
        let workspace_root_canon = config.workspace_root.canonicalize()
            .map_err(|e| AppError::Workspace(format!("Cannot resolve workspace root: {}", e)))?;
        let working_dir_canon = working_dir.canonicalize()
            .map_err(|e| AppError::Workspace(format!("Cannot resolve working directory: {}", e)))?;
        if !working_dir_canon.starts_with(&workspace_root_canon) {
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
                // On Windows cargo test writes test results to stderr, not stdout.
                // Combine both so the summary parser and UI always have the full output.
                let raw_stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let raw_stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let combined = format!("{}{}", raw_stdout, raw_stderr);
                let combined = Self::truncate_output(&combined, config.max_output_bytes);

                let exit_code = output.status.code().unwrap_or(-1) as i64;
                let status = if exit_code == 0 { "passed" } else { "failed" };

                // Save combined output to workspace log file
                let log_rel = format!("logs/{}_output.txt", execution.id);
                let workspace_id_str = working_dir_canon
                    .strip_prefix(&workspace_root_canon)
                    .ok()
                    .and_then(|p| p.components().next())
                    .map(|c| c.as_os_str().to_string_lossy().to_string())
                    .unwrap_or_default();

                if !workspace_id_str.is_empty() {
                    if let Ok(ws_path) = crate::services::workspace::WorkspaceService::validate_path(
                        config,
                        &workspace_id_str,
                        &log_rel,
                    ) {
                        if let Some(parent) = ws_path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        let _ = std::fs::write(&ws_path, &combined);
                    }
                }

                let summary = Self::build_summary(exit_code, &combined);

                TestExecutionRepo::complete(
                    db,
                    &execution.id,
                    exit_code,
                    status,
                    Some(&log_rel),
                    None,
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
                        "duration_ms": duration_ms,
                        "summary": summary
                    }),
                ).await?;

                Ok(ExecutionResult {
                    execution_id: execution.id,
                    status: status.to_string(),
                    exit_code,
                    duration_ms,
                    stdout: combined,
                    stderr: String::new(),
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

    fn build_summary(exit_code: i64, combined: &str) -> String {
        // Parse cargo test result line: "test result: FAILED. 6 passed; 5 failed; ..."
        let mut passed = 0u32;
        let mut failed = 0u32;

        for line in combined.lines() {
            // Strip ANSI escape codes before matching
            let clean: String = line.chars().filter(|c| c.is_ascii() && (*c as u8) >= 32).collect();
            if clean.contains("test result:") {
                if let Some(p) = Self::extract_count(&clean, " passed") { passed += p; }
                if let Some(f) = Self::extract_count(&clean, " failed") { failed += f; }
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
