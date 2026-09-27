use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    pub workspace_root: PathBuf,
    pub sample_project_root: PathBuf,
    pub is_demo_mode: bool,
    pub allowed_origins: Vec<String>,
    pub max_body_bytes: usize,
    pub test_timeout_secs: u64,
    pub max_output_bytes: usize,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://contractguard.db".to_string());

        let bind_addr = std::env::var("BIND_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:8080".to_string());

        let workspace_root = std::env::var("WORKSPACE_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                if std::path::Path::new("runs").exists() {
                    PathBuf::from("runs")
                } else if std::path::Path::new("../runs").exists() {
                    PathBuf::from("../runs")
                } else {
                    PathBuf::from("runs")
                }
            });

        let sample_project_root = std::env::var("SAMPLE_PROJECT_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                if std::path::Path::new("../sample_project").exists() {
                    PathBuf::from("../sample_project")
                } else if std::path::Path::new("sample_project").exists() {
                    PathBuf::from("sample_project")
                } else {
                    PathBuf::from("../sample_project")
                }
            });

        let is_demo_mode = std::env::var("DEMO_MODE")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        let allowed_origins = std::env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:5173,http://localhost:4173".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let max_body_bytes = std::env::var("MAX_BODY_BYTES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(16 * 1024 * 1024); // 16 MB

        let test_timeout_secs = std::env::var("TEST_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(120);

        let max_output_bytes = std::env::var("MAX_OUTPUT_BYTES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1024 * 1024); // 1 MB

        Ok(Config {
            database_url,
            bind_addr,
            workspace_root,
            sample_project_root,
            is_demo_mode,
            allowed_origins,
            max_body_bytes,
            test_timeout_secs,
            max_output_bytes,
        })
    }
}
