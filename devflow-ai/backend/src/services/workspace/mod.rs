use std::path::{Path, PathBuf};
use tokio::fs;

use crate::config::Config;
use crate::errors::{AppError, ApiResult};
use crate::security::path_guard::PathGuard;

pub struct WorkspaceService;

impl WorkspaceService {
    /// Creates an isolated workspace for an investigation by copying the broken sample.
    pub async fn create_workspace(config: &Config, investigation_id: &str, workspace_id: &str) -> ApiResult<PathBuf> {
        let workspace_path = config.workspace_root.join(workspace_id);

        if workspace_path.exists() {
            return Ok(workspace_path);
        }

        fs::create_dir_all(&workspace_path).await?;

        // Copy the broken fixture
        let broken_src = config.sample_project_root.join("broken");
        if broken_src.exists() {
            Self::copy_dir_all(&broken_src, &workspace_path.join("broken")).await?;
        }

        // Create the bob artifacts directory
        fs::create_dir_all(workspace_path.join("bob_artifacts")).await?;

        // Create a workspace metadata file
        let meta = serde_json::json!({
            "investigation_id": investigation_id,
            "workspace_id": workspace_id,
            "created_at": chrono::Utc::now().to_rfc3339(),
        });
        fs::write(
            workspace_path.join("workspace.json"),
            serde_json::to_string_pretty(&meta).unwrap_or_default(),
        ).await?;

        Ok(workspace_path)
    }

    pub fn workspace_path(config: &Config, workspace_id: &str) -> PathBuf {
        config.workspace_root.join(workspace_id)
    }

    /// Validate that a path is within the workspace root (no path traversal).
    pub fn validate_path(config: &Config, workspace_id: &str, relative: &str) -> ApiResult<PathBuf> {
        PathGuard::validate_workspace_path(config, workspace_id, relative)
    }

    pub async fn read_artifact(config: &Config, workspace_id: &str, relative: &str) -> ApiResult<String> {
        let path = Self::validate_path(config, workspace_id, relative)?;
        if !path.exists() {
            return Err(AppError::NotFound(format!("Artifact not found: {}", relative)));
        }
        let content = fs::read_to_string(&path).await?;
        Ok(content)
    }

    /// Synchronous version for use in non-async map/closure contexts.
    pub fn read_artifact_sync(config: &Config, workspace_id: &str, relative: &str) -> Option<String> {
        let path = Self::validate_path(config, workspace_id, relative).ok()?;
        std::fs::read_to_string(&path).ok()
    }

    pub async fn write_artifact(config: &Config, workspace_id: &str, relative: &str, content: &str) -> ApiResult<()> {
        let path = Self::validate_path(config, workspace_id, relative)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(&path, content).await?;
        Ok(())
    }

    async fn copy_dir_all(src: &Path, dst: &Path) -> ApiResult<()> {
        fs::create_dir_all(dst).await?;
        let mut entries = fs::read_dir(src).await?;
        while let Some(entry) = entries.next_entry().await? {
            let entry_type = entry.file_type().await?;
            let dst_path = dst.join(entry.file_name());
            if entry_type.is_dir() {
                Box::pin(Self::copy_dir_all(&entry.path(), &dst_path)).await?;
            } else if entry_type.is_file() {
                fs::copy(entry.path(), &dst_path).await?;
            }
            // Skip symlinks intentionally
        }
        Ok(())
    }
}
