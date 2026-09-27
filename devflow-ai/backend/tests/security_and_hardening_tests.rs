use axum::extract::{Path, State};
use axum::Json;
use contractguard_lib::config::Config;
use contractguard_lib::errors::AppError;
use contractguard_lib::handlers::{
    approvals::create_approval,
    artifacts::sync_artifacts,
    investigations::{create_investigation, get_investigation, run_baseline},
    metrics::{get_metrics, record_manual_baseline, ManualBaselineRequest},
    reports::get_report,
    verification::run_verification,
};
use contractguard_lib::models::investigation::{ApprovalRequest, CreateInvestigationRequest};
use contractguard_lib::repositories::InvestigationRepo;
use contractguard_lib::services::investigation::InvestigationService;
use contractguard_lib::services::verification::CMD_RUST_BACKEND_TESTS;
use contractguard_lib::services::workspace::WorkspaceService;
use contractguard_lib::state::AppState;
use sqlx::sqlite::SqlitePoolOptions;
use std::path::PathBuf;
use std::sync::Arc;

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

fn test_config(test_name: &str, is_demo_mode: bool) -> (Config, PathBuf) {
    let test_dir = std::env::temp_dir().join(format!("contractguard_sec_test_{}", test_name));
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
        is_demo_mode,
        allowed_origins: vec!["http://localhost:5173".into()],
        max_body_bytes: 1024 * 1024,
        test_timeout_secs: 120,
        max_output_bytes: 1024 * 1024,
        static_dir: None,
    };

    (config, test_dir)
}

#[tokio::test]
async fn test_demo_mode_blocks_all_mutations() {
    let db = setup_test_db().await;
    let (mut config, _test_dir) = test_config("demo_mode_blocks", false);

    // Create an investigation initially in non-demo mode
    let inv = InvestigationService::create(
        &db,
        &config,
        CreateInvestigationRequest {
            title: "Demo Mode Test Inv".into(),
            description: "Testing demo mode protection".into(),
            project_id: "uploadlab".into(),
            scenario_id: "uploadlab_bug_001_002".into(),
            expected_behavior: None,
            observed_behavior: None,
            reproduction_steps: None,
        },
    )
    .await
    .unwrap();

    // Switch to demo mode
    config.is_demo_mode = true;
    let state = Arc::new(AppState::new(db.clone(), config.clone()));

    // 1. Attempt create_investigation -> MUST FAIL with DemoModeMutation
    let create_res = create_investigation(
        State(state.clone()),
        Json(CreateInvestigationRequest {
            title: "Disallowed Create".into(),
            description: "Should fail in demo mode".into(),
            project_id: "uploadlab".into(),
            scenario_id: "uploadlab_bug_001_002".into(),
            expected_behavior: None,
            observed_behavior: None,
            reproduction_steps: None,
        }),
    )
    .await;
    assert!(matches!(create_res, Err(AppError::DemoModeMutation)));

    // 2. Attempt run_baseline -> MUST FAIL with DemoModeMutation
    let baseline_res = run_baseline(State(state.clone()), Path(inv.id.clone())).await;
    assert!(matches!(baseline_res, Err(AppError::DemoModeMutation)));

    // 3. Attempt sync_artifacts -> MUST FAIL with DemoModeMutation
    let sync_res = sync_artifacts(State(state.clone()), Path(inv.id.clone())).await;
    assert!(matches!(sync_res, Err(AppError::DemoModeMutation)));

    // 4. Attempt create_approval -> MUST FAIL with DemoModeMutation
    let approval_res = create_approval(
        State(state.clone()),
        Path(inv.id.clone()),
        Json(ApprovalRequest {
            decision: "approved".into(),
            comment: Some("Test".into()),
            approver_id: Some("sec_admin".into()),
        }),
    )
    .await;
    assert!(matches!(approval_res, Err(AppError::DemoModeMutation)));

    // 5. Attempt run_verification -> MUST FAIL with DemoModeMutation
    let verif_res = run_verification(State(state.clone()), Path(inv.id.clone())).await;
    assert!(matches!(verif_res, Err(AppError::DemoModeMutation)));

    // 6. Attempt record_manual_baseline -> MUST FAIL with DemoModeMutation
    let manual_res = record_manual_baseline(
        State(state.clone()),
        Path(inv.id.clone()),
        Json(ManualBaselineRequest {
            scenario_id: inv.scenario_id.clone(),
            start_time: "2026-09-27T10:00:00Z".into(),
            finish_time: None,
            active_minutes: Some(30),
            completed_steps: None,
            final_test_status: None,
        }),
    )
    .await;
    assert!(matches!(manual_res, Err(AppError::DemoModeMutation)));

    // 7. Verify READ-ONLY operations still succeed in demo mode
    let get_res = get_investigation(State(state.clone()), Path(inv.id.clone())).await;
    assert!(get_res.is_ok(), "Read operation must succeed in demo mode");

    let report_res = get_report(State(state.clone()), Path(inv.id.clone())).await;
    assert!(
        report_res.is_ok(),
        "Report generation must succeed in demo mode"
    );

    let metrics_res = get_metrics(State(state.clone()), Path(inv.id.clone())).await;
    assert!(
        metrics_res.is_ok(),
        "Metrics read must succeed in demo mode"
    );
}

#[tokio::test]
async fn test_path_traversal_rejection() {
    let (config, _test_dir) = test_config("path_traversal", false);
    let workspace_id = "test-ws-uuid";

    // Create workspace dir
    let ws_path = config.workspace_root.join(workspace_id);
    std::fs::create_dir_all(&ws_path).unwrap();

    // 1. Relative dot-dot traversal
    let res1 = WorkspaceService::validate_path(&config, workspace_id, "../../../etc/passwd");
    assert!(matches!(res1, Err(AppError::PathTraversal)));

    let res2 = WorkspaceService::validate_path(&config, workspace_id, "..\\..\\windows\\system32");
    assert!(matches!(res2, Err(AppError::PathTraversal)));

    // 2. Absolute path traversal
    let res3 = WorkspaceService::validate_path(&config, workspace_id, "/etc/shadow");
    assert!(matches!(res3, Err(AppError::PathTraversal)));

    #[cfg(windows)]
    {
        let res4 = WorkspaceService::validate_path(&config, workspace_id, "C:\\Windows\\cmd.exe");
        assert!(matches!(res4, Err(AppError::PathTraversal)));
    }

    // 3. Valid in-workspace path
    let res5 =
        WorkspaceService::validate_path(&config, workspace_id, "bob_artifacts/diagnosis.json");
    assert!(res5.is_ok(), "In-workspace relative path must be accepted");
}

#[tokio::test]
async fn test_error_response_sanitization() {
    use axum::response::IntoResponse;

    // Workspace error containing an internal path
    let err =
        AppError::Workspace("Failed to open C:\\Users\\secret\\internal\\file.txt".to_string());
    let resp = err.into_response();
    assert_eq!(resp.status(), axum::http::StatusCode::INTERNAL_SERVER_ERROR);

    // Verify sanitized error message
    let body_bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);
    assert!(
        !body_str.contains("C:\\Users\\secret"),
        "Internal path must not be in response body"
    );
    assert!(
        !body_str.contains("internal"),
        "Internal keywords must not be in response body"
    );
    assert!(
        body_str.contains("Workspace error"),
        "Generic error message expected"
    );
}

#[tokio::test]
async fn test_state_preservation_on_artifact_sync_failure() {
    let db = setup_test_db().await;
    let (config, _test_dir) = test_config("state_preservation", false);
    let state = Arc::new(AppState::new(db.clone(), config.clone()));

    let inv = InvestigationService::create(
        &db,
        &config,
        CreateInvestigationRequest {
            title: "Sync Failure Test".into(),
            description: "State preservation check".into(),
            project_id: "uploadlab".into(),
            scenario_id: "uploadlab_bug_001_002".into(),
            expected_behavior: None,
            observed_behavior: None,
            reproduction_steps: None,
        },
    )
    .await
    .unwrap();

    assert_eq!(inv.status, "WORKSPACE_READY");

    // Create bob_artifacts with invalid JSON
    let ws_dir = config
        .workspace_root
        .join(&inv.workspace_id)
        .join("bob_artifacts");
    std::fs::create_dir_all(&ws_dir).unwrap();
    std::fs::write(ws_dir.join("diagnosis.json"), "{ invalid JSON content...").unwrap();

    // Call sync_artifacts - this should report errors or fail
    let sync_res = sync_artifacts(State(state.clone()), Path(inv.id.clone()))
        .await
        .unwrap();
    let sync_data = sync_res.0;

    // Verify errors were captured
    let errors = sync_data["errors"].as_array().unwrap();
    assert!(
        !errors.is_empty(),
        "Errors must be reported for invalid JSON"
    );

    // Verify status was NOT advanced (preserved at WORKSPACE_READY)
    let current_inv = InvestigationRepo::get_by_id(&db, &inv.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        current_inv.status, "WORKSPACE_READY",
        "Investigation status must be preserved when artifact sync encounters validation errors"
    );
}

#[tokio::test]
async fn test_allowed_commands_constant() {
    // Verify predefined command identifier matches required specification
    assert_eq!(CMD_RUST_BACKEND_TESTS, "rust_backend_tests");
}
