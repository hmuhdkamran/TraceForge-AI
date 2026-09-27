use std::path::PathBuf;
use contractguard_lib::config::Config;
use contractguard_lib::models::investigation::Investigation;
use contractguard_lib::services::prompts::PromptService;

fn sample_config() -> Config {
    Config {
        database_url: "sqlite::memory:".into(),
        bind_addr: "127.0.0.1:8080".into(),
        workspace_root: PathBuf::from("runs"),
        sample_project_root: PathBuf::from("../sample_project"),
        is_demo_mode: false,
        allowed_origins: vec!["http://localhost:5173".into()],
        max_body_bytes: 1024 * 1024,
        test_timeout_secs: 60,
        max_output_bytes: 1024 * 1024,
    }
}

fn sample_investigation() -> Investigation {
    Investigation {
        id: "inv-test-123".into(),
        title: "Multi-file upload bug".into(),
        description: "Backend only processes the first file".into(),
        project_id: "uploadlab".into(),
        scenario_id: "multi-file-upload-failure".into(),
        status: "WORKSPACE_READY".into(),
        workspace_id: "ws-test-123".into(),
        created_at: "2026-09-27T10:00:00Z".into(),
        updated_at: "2026-09-27T10:00:00Z".into(),
        approved_at: None,
        completed_at: None,
        failure_reason: None,
    }
}

#[test]
fn test_investigation_prompt_generation() {
    let config = sample_config();
    let inv = sample_investigation();

    let prompt = PromptService::generate_investigation_prompt(&inv, &config);

    assert!(prompt.contains("inv-test-123"));
    assert!(prompt.contains("Multi-file upload bug"));
    assert!(prompt.contains("Backend only processes the first file"));
    assert!(prompt.contains("Step 1 — Read the project documentation"));
    assert!(prompt.contains("Step 2 — Investigate the frontend"));
    assert!(prompt.contains("Step 3 — Investigate the backend"));
    assert!(prompt.contains("diagnosis.json"));
    assert!(prompt.contains("investigation_summary.md"));
    assert!(prompt.contains("fix_plan.md"));
    assert!(prompt.contains("Do NOT edit any files under"));
}

#[test]
fn test_implementation_prompt_generation() {
    let config = sample_config();
    let mut inv = sample_investigation();
    inv.status = "APPROVED".into();
    inv.approved_at = Some("2026-09-27T10:30:00Z".into());

    let plan_hash = "sha256:abc123def456";
    let fix_plan = "1. Fix field name to 'files'\n2. Iterate through all multipart fields";

    let prompt = PromptService::generate_implementation_prompt(&inv, &config, plan_hash, fix_plan);

    assert!(prompt.contains("inv-test-123"));
    assert!(prompt.contains(plan_hash));
    assert!(prompt.contains("Fix field name to 'files'"));
    assert!(prompt.contains("implementation_summary.md"));
    assert!(prompt.contains("changed_files.json"));
    assert!(prompt.contains("review.md"));
}
