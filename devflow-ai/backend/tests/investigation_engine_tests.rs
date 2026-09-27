use contractguard_lib::config::Config;
use contractguard_lib::models::investigation::CreateInvestigationRequest;
use contractguard_lib::repositories::{AuditRepo, InvestigationRepo};
use contractguard_lib::services::investigation::InvestigationService;
use sqlx::sqlite::SqlitePoolOptions;
use std::path::PathBuf;

async fn setup_test_db() -> sqlx::SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory sqlite pool");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

fn test_config(test_name: &str) -> (Config, PathBuf) {
    let test_dir = std::env::temp_dir().join(format!("contractguard_test_{}", test_name));
    let _ = std::fs::remove_dir_all(&test_dir);
    std::fs::create_dir_all(&test_dir).expect("Failed to create test dir");

    let sample_root = if PathBuf::from("sample_project").exists() {
        PathBuf::from("sample_project")
    } else if PathBuf::from("../sample_project").exists() {
        PathBuf::from("../sample_project")
    } else {
        PathBuf::from("../../sample_project")
    };

    let config = Config {
        database_url: "sqlite::memory:".into(),
        bind_addr: "127.0.0.1:8080".into(),
        workspace_root: test_dir.clone(),
        sample_project_root: sample_root,
        is_demo_mode: false,
        allowed_origins: vec!["http://localhost:5173".into()],
        max_body_bytes: 1024 * 1024,
        test_timeout_secs: 60,
        max_output_bytes: 1024 * 1024,
        static_dir: None,
    };

    (config, test_dir)
}

#[tokio::test]
async fn test_investigation_creation_and_workspace_isolation() {
    let db = setup_test_db().await;
    let (config, test_dir) = test_config("create_isolation");

    let req = CreateInvestigationRequest {
        title: "E2E Investigation Test".into(),
        description: "Testing automated workspace isolation".into(),
        project_id: "uploadlab".into(),
        scenario_id: "multi-file-upload-failure".into(),
        expected_behavior: Some("Should process all files".into()),
        observed_behavior: Some("Only first file processed".into()),
        reproduction_steps: Some("1. Upload multiple files".into()),
    };

    let investigation = InvestigationService::create(&db, &config, req)
        .await
        .expect("Investigation creation failed");

    // 1. Assert DB record
    assert_eq!(investigation.title, "E2E Investigation Test");
    assert_eq!(investigation.status, "WORKSPACE_READY");
    assert!(!investigation.workspace_id.is_empty());

    // 2. Assert Workspace Isolation
    let ws_dir = test_dir.join(&investigation.workspace_id);
    assert!(ws_dir.exists(), "Workspace dir should exist");
    assert!(
        ws_dir.join("workspace.json").exists(),
        "workspace.json metadata should exist"
    );
    assert!(
        ws_dir.join("broken").exists(),
        "broken fixture should be copied"
    );
    assert!(
        ws_dir.join("bob_artifacts").exists(),
        "bob_artifacts dir should exist"
    );

    // 3. Assert Audit Log Events
    let audit_events = AuditRepo::list_for_investigation(&db, &investigation.id)
        .await
        .expect("Failed to fetch audit events");
    assert!(audit_events
        .iter()
        .any(|e| e.event_type == "investigation.created"));
    assert!(audit_events
        .iter()
        .any(|e| e.event_type == "workspace.created"));

    // 4. Assert State Machine Transitions
    InvestigationService::transition_status(&db, &investigation, "BASELINE_RUNNING")
        .await
        .expect("Transition to BASELINE_RUNNING should succeed");

    let updated = InvestigationRepo::get_by_id(&db, &investigation.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.status, "BASELINE_RUNNING");

    InvestigationService::transition_status(&db, &updated, "BASELINE_CAPTURED")
        .await
        .expect("Transition to BASELINE_CAPTURED should succeed");

    let updated2 = InvestigationRepo::get_by_id(&db, &investigation.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated2.status, "BASELINE_CAPTURED");

    InvestigationService::transition_status(&db, &updated2, "AWAITING_BOB_INVESTIGATION")
        .await
        .expect("Transition to AWAITING_BOB_INVESTIGATION should succeed");

    // Cleanup
    let _ = std::fs::remove_dir_all(&test_dir);
}

#[tokio::test]
async fn test_invalid_investigation_creation_rejected() {
    let db = setup_test_db().await;
    let (config, test_dir) = test_config("invalid_input");

    // Missing title
    let req = CreateInvestigationRequest {
        title: "   ".into(),
        description: "Valid description".into(),
        project_id: "uploadlab".into(),
        scenario_id: "scenario-1".into(),
        expected_behavior: None,
        observed_behavior: None,
        reproduction_steps: None,
    };
    let result = InvestigationService::create(&db, &config, req).await;
    assert!(result.is_err());

    // Missing description
    let req2 = CreateInvestigationRequest {
        title: "Valid title".into(),
        description: "   ".into(),
        project_id: "uploadlab".into(),
        scenario_id: "scenario-1".into(),
        expected_behavior: None,
        observed_behavior: None,
        reproduction_steps: None,
    };
    let result2 = InvestigationService::create(&db, &config, req2).await;
    assert!(result2.is_err());

    let _ = std::fs::remove_dir_all(&test_dir);
}
