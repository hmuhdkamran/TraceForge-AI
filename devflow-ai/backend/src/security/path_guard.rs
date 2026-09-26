use std::path::PathBuf;

use crate::config::Config;
use crate::errors::{AppError, ApiResult};

pub struct PathGuard;

impl PathGuard {
    /// Ensure relative path stays within workspace root; reject traversal and symlinks.
    pub fn validate_workspace_path(config: &Config, workspace_id: &str, relative: &str) -> ApiResult<PathBuf> {
        // Reject obvious traversal patterns before canonicalization
        if relative.contains("..") {
            return Err(AppError::PathTraversal);
        }

        // Sanitize path separators
        let clean = relative.replace('\\', "/");
        if clean.starts_with('/') {
            return Err(AppError::PathTraversal);
        }

        let base = config.workspace_root.join(workspace_id);
        let candidate = base.join(&clean);

        // Check that the resolved path starts with base
        // We use starts_with on the string components rather than canonicalize
        // because the path may not yet exist (write case).
        let base_str = base.to_string_lossy();
        let candidate_str = candidate.to_string_lossy();

        if !candidate_str.starts_with(base_str.as_ref()) {
            return Err(AppError::PathTraversal);
        }

        // If the path exists, verify it's not a symlink
        if candidate.exists() {
            let meta = std::fs::symlink_metadata(&candidate)?;
            if meta.file_type().is_symlink() {
                return Err(AppError::PathTraversal);
            }
        }

        Ok(candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_config(root: &str) -> Config {
        Config {
            database_url: "sqlite::memory:".into(),
            bind_addr: "127.0.0.1:8080".into(),
            workspace_root: PathBuf::from(root),
            sample_project_root: PathBuf::from("../sample_project"),
            is_demo_mode: false,
            allowed_origins: vec![],
            max_body_bytes: 1024 * 1024,
            test_timeout_secs: 60,
            max_output_bytes: 1024 * 1024,
        }
    }

    #[test]
    fn accepts_valid_path() {
        let config = make_config("/tmp/runs");
        let result = PathGuard::validate_workspace_path(&config, "ws1", "bob_artifacts/diagnosis.json");
        assert!(result.is_ok());
    }

    #[test]
    fn rejects_traversal() {
        let config = make_config("/tmp/runs");
        let result = PathGuard::validate_workspace_path(&config, "ws1", "../other/file");
        assert!(matches!(result, Err(AppError::PathTraversal)));
    }

    #[test]
    fn rejects_absolute_path() {
        let config = make_config("/tmp/runs");
        let result = PathGuard::validate_workspace_path(&config, "ws1", "/etc/passwd");
        assert!(matches!(result, Err(AppError::PathTraversal)));
    }
}
