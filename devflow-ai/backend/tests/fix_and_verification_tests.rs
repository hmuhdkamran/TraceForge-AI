use axum::extract::{Path, State};
use axum::Json;
use contractguard_lib::config::Config;
use contractguard_lib::handlers::{
    approvals::{create_approval, get_plan},
    artifacts::sync_artifacts,
    investigations::get_bob_implementation_prompt,
    verification::{get_diff, get_verification, run_verification},
};
use contractguard_lib::models::investigation::{ApprovalRequest, CreateInvestigationRequest};
use contractguard_lib::repositories::{AuditRepo, InvestigationRepo};
use contractguard_lib::services::investigation::InvestigationService;
use contractguard_lib::services::verification::VerificationService;
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

fn test_config(test_name: &str) -> (Config, PathBuf) {
    let test_dir = std::env::temp_dir().join(format!("contractguard_fix_test_{}", test_name));
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
        test_timeout_secs: 120,
        max_output_bytes: 1024 * 1024,
        static_dir: None,
    };

    (config, test_dir)
}

#[tokio::test]
async fn test_human_approval_workflow() {
    let db = setup_test_db().await;
    let (config, test_dir) = test_config("approval_flow");
    let shared_state = Arc::new(AppState::new(db.clone(), config.clone()));

    let req = CreateInvestigationRequest {
        title: "Approval Workflow Test".into(),
        description: "Testing human approval and plan locking".into(),
        project_id: "uploadlab".into(),
        scenario_id: "multi-file-upload-failure".into(),
        expected_behavior: None,
        observed_behavior: None,
        reproduction_steps: None,
    };

    let inv = InvestigationService::create(&db, &config, req)
        .await
        .unwrap();
    let ws_path = config.workspace_root.join(&inv.workspace_id);
    let bob_artifacts_dir = ws_path.join("bob_artifacts");
    std::fs::create_dir_all(&bob_artifacts_dir).unwrap();

    let fix_plan_content = "# Minimal Fix Plan\n\n1. Replace `if let Some(field)` with `while let Some(field)`\n2. Update frontend field to 'files'\n";
    std::fs::write(bob_artifacts_dir.join("fix_plan.md"), fix_plan_content).unwrap();

    let diagnosis_json = serde_json::json!({
        "schema_version": 1,
        "investigation_id": inv.id,
        "project_id": "uploadlab",
        "summary": "Root cause confirmed",
        "findings": [
            {
                "id": "FINDING-001",
                "finding_type": "contract_mismatch",
                "title": "Field mismatch",
                "description": "Mismatch"
            }
        ]
    });
    std::fs::write(
        bob_artifacts_dir.join("diagnosis.json"),
        serde_json::to_string_pretty(&diagnosis_json).unwrap(),
    )
    .unwrap();

    // Sync artifacts to advance state to AWAITING_APPROVAL
    let _ = sync_artifacts(State(shared_state.clone()), Path(inv.id.clone()))
        .await
        .unwrap();

    let plan_res = get_plan(State(shared_state.clone()), Path(inv.id.clone()))
        .await
        .unwrap();
    let plan_val = plan_res.0;
    assert_eq!(plan_val["fix_plan"], fix_plan_content);
    assert!(!plan_val["plan_hash"].as_str().unwrap().is_empty());
    let expected_hash = plan_val["plan_hash"].as_str().unwrap().to_string();

    // 1. Rejection decision
    let reject_req = ApprovalRequest {
        decision: "rejected".into(),
        comment: Some("Need more test coverage first".into()),
        approver_id: Some("reviewer-alice".into()),
    };
    let reject_res = create_approval(
        State(shared_state.clone()),
        Path(inv.id.clone()),
        Json(reject_req),
    )
    .await
    .unwrap();
    assert_eq!(reject_res.0["approval"]["decision"], "rejected");

    let inv_after_reject = InvestigationRepo::get_by_id(&db, &inv.id)
        .await
        .unwrap()
        .unwrap();
    assert!(inv_after_reject.approved_at.is_none());

    // 2. Approval decision
    let approve_req = ApprovalRequest {
        decision: "approved".into(),
        comment: Some("Looks solid, proceed to implementation".into()),
        approver_id: Some("tech-lead-bob".into()),
    };
    let approve_res = create_approval(
        State(shared_state.clone()),
        Path(inv.id.clone()),
        Json(approve_req),
    )
    .await
    .unwrap();
    assert_eq!(approve_res.0["approval"]["decision"], "approved");
    assert_eq!(approve_res.0["approval"]["plan_hash"], expected_hash);

    let inv_after_approve = InvestigationRepo::get_by_id(&db, &inv.id)
        .await
        .unwrap()
        .unwrap();
    assert!(inv_after_approve.approved_at.is_some());
    assert_eq!(inv_after_approve.status, "APPROVED");

    // Verify audit trail
    let audit_events = AuditRepo::list_for_investigation(&db, &inv.id)
        .await
        .unwrap();
    assert!(audit_events
        .iter()
        .any(|e| e.event_type == "approval.rejected"));
    assert!(audit_events
        .iter()
        .any(|e| e.event_type == "approval.approved"));
    assert!(audit_events
        .iter()
        .any(|e| e.event_type == "status.approved"));

    // Verify implementation prompt is unlocked
    let prompt_res =
        get_bob_implementation_prompt(State(shared_state.clone()), Path(inv.id.clone()))
            .await
            .unwrap();
    assert_eq!(prompt_res.0["plan_hash"], expected_hash);
    assert!(prompt_res.0["prompt"]
        .as_str()
        .unwrap()
        .contains(&expected_hash));

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[tokio::test]
async fn test_empty_plan_approval_rejected() {
    let db = setup_test_db().await;
    let (config, test_dir) = test_config("empty_plan");
    let shared_state = Arc::new(AppState::new(db.clone(), config.clone()));

    let req = CreateInvestigationRequest {
        title: "Empty Plan Test".into(),
        description: "Checking rejection when fix_plan.md is missing".into(),
        project_id: "uploadlab".into(),
        scenario_id: "multi-file-upload-failure".into(),
        expected_behavior: None,
        observed_behavior: None,
        reproduction_steps: None,
    };

    let inv = InvestigationService::create(&db, &config, req)
        .await
        .unwrap();

    let approve_req = ApprovalRequest {
        decision: "approved".into(),
        comment: None,
        approver_id: None,
    };

    // Investigation status is WORKSPACE_READY, not AWAITING_APPROVAL
    let res = create_approval(
        State(shared_state.clone()),
        Path(inv.id.clone()),
        Json(approve_req),
    )
    .await;
    assert!(
        res.is_err(),
        "Approval without AWAITING_APPROVAL status must fail"
    );

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[tokio::test]
async fn test_verification_and_prompt_require_approval() {
    let db = setup_test_db().await;
    let (config, test_dir) = test_config("approval_required");
    let shared_state = Arc::new(AppState::new(db.clone(), config.clone()));

    let req = CreateInvestigationRequest {
        title: "Approval Required Test".into(),
        description: "Verification must be blocked until approval".into(),
        project_id: "uploadlab".into(),
        scenario_id: "multi-file-upload-failure".into(),
        expected_behavior: None,
        observed_behavior: None,
        reproduction_steps: None,
    };

    let inv = InvestigationService::create(&db, &config, req)
        .await
        .unwrap();

    // 1. Implementation prompt blocked
    let prompt_err =
        get_bob_implementation_prompt(State(shared_state.clone()), Path(inv.id.clone())).await;
    assert!(prompt_err.is_err());

    // 2. Verification execution blocked
    let verify_err = run_verification(State(shared_state.clone()), Path(inv.id.clone())).await;
    assert!(verify_err.is_err());

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[tokio::test]
async fn test_end_to_end_fix_and_verification_pipeline() {
    let db = setup_test_db().await;
    let (config, test_dir) = test_config("e2e_fix_verification");
    let shared_state = Arc::new(AppState::new(db.clone(), config.clone()));

    let req = CreateInvestigationRequest {
        title: "E2E Bug Fix & Verification Pipeline".into(),
        description: "Full vertical slice: Baseline failure -> Bob Investigation -> Approval -> Fix -> Verified".into(),
        project_id: "uploadlab".into(),
        scenario_id: "multi-file-upload-failure".into(),
        expected_behavior: Some("All multipart files processed".into()),
        observed_behavior: Some("Only first file processed".into()),
        reproduction_steps: Some("Upload 2 or 3 files".into()),
    };

    let inv = InvestigationService::create(&db, &config, req)
        .await
        .unwrap();

    // -------------------------------------------------------------------------
    // Phase 1: Run Baseline on Broken Workspace (Expects Failure)
    // -------------------------------------------------------------------------
    let baseline_result = VerificationService::run_baseline(&db, &config, &inv, "baseline")
        .await
        .expect("Baseline run failed");

    assert_ne!(
        baseline_result.exit_code, 0,
        "Baseline tests on broken fixture should fail"
    );
    assert_eq!(baseline_result.status, "failed");
    assert!(
        baseline_result.summary.contains("passed") && baseline_result.summary.contains("failed"),
        "Summary should record passed and failed tests: {}",
        baseline_result.summary
    );

    // -------------------------------------------------------------------------
    // Phase 2: Bob Investigation Artifacts & Approval
    // -------------------------------------------------------------------------
    let ws_path = config.workspace_root.join(&inv.workspace_id);
    let bob_artifacts_dir = ws_path.join("bob_artifacts");
    std::fs::create_dir_all(&bob_artifacts_dir).unwrap();

    let fix_plan_content = "# Fix Plan\n\n1. Modify `broken/backend/src/main.rs` to loop through all multipart fields\n2. Modify `broken/frontend/src/api.ts` to use 'files'\n";
    std::fs::write(bob_artifacts_dir.join("fix_plan.md"), fix_plan_content).unwrap();

    let diagnosis_json = serde_json::json!({
        "schema_version": 1,
        "investigation_id": inv.id,
        "project_id": "uploadlab",
        "summary": "Confirmed BUG-001 (frontend field) and BUG-002 (backend loop break)",
        "findings": [
            {
                "id": "FINDING-001",
                "finding_type": "contract_mismatch",
                "title": "Frontend field mismatch",
                "description": "Uses 'file' instead of 'files'",
                "source_references": [
                    {
                        "relative_path": "broken/frontend/src/api.ts",
                        "start_line": 20,
                        "end_line": 28
                    }
                ]
            },
            {
                "id": "FINDING-002",
                "finding_type": "backend_defect",
                "title": "Backend early termination",
                "description": "Does not iterate over all fields",
                "source_references": [
                    {
                        "relative_path": "broken/backend/src/main.rs",
                        "start_line": 40,
                        "end_line": 65
                    }
                ]
            }
        ]
    });
    std::fs::write(
        bob_artifacts_dir.join("diagnosis.json"),
        serde_json::to_string_pretty(&diagnosis_json).unwrap(),
    )
    .unwrap();

    let _ = sync_artifacts(State(shared_state.clone()), Path(inv.id.clone()))
        .await
        .unwrap();

    // Human Approval
    let approve_req = ApprovalRequest {
        decision: "approved".into(),
        comment: Some("Authorized for execution".into()),
        approver_id: Some("lead-dev".into()),
    };
    let _ = create_approval(
        State(shared_state.clone()),
        Path(inv.id.clone()),
        Json(approve_req),
    )
    .await
    .unwrap();

    let approved_inv = InvestigationRepo::get_by_id(&db, &inv.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(approved_inv.status, "APPROVED");

    // -------------------------------------------------------------------------
    // Phase 3: Bob Implementation (Fix Code in Isolated Workspace)
    // -------------------------------------------------------------------------
    let backend_main = ws_path
        .join("broken")
        .join("backend")
        .join("src")
        .join("main.rs");
    let original_main_code =
        std::fs::read_to_string(&backend_main).expect("Failed to read backend main.rs");

    // Apply the fix for BUG-002: replace `if let Some(field)` with `while let Some(field)` and check uploaded.is_empty()
    // Also update regression test `test_invalid_batch_behavior_bug002` to assert 422 rejection on invalid batch
    let norm_main_code = original_main_code.replace("\r\n", "\n");
    assert!(norm_main_code.contains("if let Some(field) = multipart.next_field().await"));
    let fixed_main_code = norm_main_code
        .replace(
            "if let Some(field) = multipart.next_field().await",
            "while let Some(field) = multipart.next_field().await",
        )
        .replace(
            "} else {\n        // No files uploaded\n        return Err((\n            StatusCode::BAD_REQUEST,\n            Json(ErrorResponse { error: \"No files uploaded\".to_string() }),\n        ));\n    }",
            "}\n\n    if uploaded.is_empty() {\n        return Err((\n            StatusCode::BAD_REQUEST,\n            Json(ErrorResponse { error: \"No files uploaded\".to_string() }),\n        ));\n    }",
        )
        .replace(
            "let bytes = resp.into_body().collect().await.unwrap().to_bytes();\n        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();\n        // On broken fixture: count=1 (only first file processed, second ignored)\n        // BUG-002 means we never detect the invalid second file\n        assert_eq!(json[\"count\"], 1, \"BUG-002: Backend only processes first file, missing invalid second file detection\");",
            "assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);\n        let bytes = resp.into_body().collect().await.unwrap().to_bytes();\n        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();\n        assert!(json[\"error\"].as_str().unwrap().contains(\"Unsupported file format\"));",
        );

    std::fs::write(&backend_main, &fixed_main_code).expect("Failed to write fixed backend main.rs");

    // Apply the fix for BUG-001 in frontend: replace 'file' with 'files'
    let frontend_api = ws_path
        .join("broken")
        .join("frontend")
        .join("src")
        .join("api.ts");
    if frontend_api.exists() {
        let original_api_code = std::fs::read_to_string(&frontend_api).unwrap();
        let fixed_api_code =
            original_api_code.replace("formData.append('file',", "formData.append('files',");
        std::fs::write(&frontend_api, &fixed_api_code).unwrap();
    }

    // Write implementation artifacts
    std::fs::write(
        bob_artifacts_dir.join("implementation_summary.md"),
        "# Implementation Summary\n\n- Fixed BUG-002 by converting `if let Some(field)` to `while let Some(field)` loop\n- Fixed BUG-001 by changing multipart field to 'files'\n",
    ).unwrap();

    let plan = get_plan(State(shared_state.clone()), Path(inv.id.clone()))
        .await
        .unwrap();
    let plan_hash = plan.0["plan_hash"].as_str().unwrap();

    let changed_files_json = serde_json::json!({
        "schema_version": 1,
        "investigation_id": inv.id,
        "plan_hash": plan_hash,
        "files": [
            {
                "relative_path": "broken/backend/src/main.rs",
                "change_type": "modified",
                "description": "Process all multipart files in loop"
            },
            {
                "relative_path": "broken/frontend/src/api.ts",
                "change_type": "modified",
                "description": "Send multipart field with name 'files'"
            }
        ]
    });
    std::fs::write(
        bob_artifacts_dir.join("changed_files.json"),
        serde_json::to_string_pretty(&changed_files_json).unwrap(),
    )
    .unwrap();

    // -------------------------------------------------------------------------
    // Phase 4: Independent Verification Execution (Expects Success!)
    // -------------------------------------------------------------------------
    let verification_result = VerificationService::run_verification(&db, &config, &approved_inv)
        .await
        .expect("Verification run failed");

    assert_eq!(
        verification_result.exit_code, 0,
        "Verification tests should PASS after fix"
    );
    assert_eq!(verification_result.status, "passed");
    assert!(
        verification_result.summary.contains("11 passed, 0 failed")
            || verification_result.summary.contains("passed"),
        "Summary should record all tests passing: {}",
        verification_result.summary
    );

    // -------------------------------------------------------------------------
    // Phase 5: Before / After Comparison & Diff Endpoint Inspection
    // -------------------------------------------------------------------------
    let verification_status = get_verification(State(shared_state.clone()), Path(inv.id.clone()))
        .await
        .unwrap();
    let val = verification_status.0;

    let baseline_exec = &val["baseline"];
    let verif_exec = &val["verification"];

    assert_eq!(baseline_exec["status"], "failed");
    assert_ne!(baseline_exec["exit_code"], 0);

    assert_eq!(verif_exec["status"], "passed");
    assert_eq!(verif_exec["exit_code"], 0);

    // Inspect Diff endpoint
    let diff_res = get_diff(State(shared_state.clone()), Path(inv.id.clone()))
        .await
        .unwrap();
    let diff_val = diff_res.0;
    assert!(diff_val["diff"]
        .as_str()
        .unwrap()
        .contains("while let Some(field)"));

    // -------------------------------------------------------------------------
    // Phase 6: Assert Immutability of Original Sample Project
    // -------------------------------------------------------------------------
    let original_sample_main = config
        .sample_project_root
        .join("broken")
        .join("backend")
        .join("src")
        .join("main.rs");
    let original_sample_code = std::fs::read_to_string(&original_sample_main).unwrap();
    assert!(
        original_sample_code.contains("if let Some(field) = multipart.next_field().await"),
        "Original sample project must remain untouched and intentionally broken!"
    );

    let _ = std::fs::remove_dir_all(&test_dir);
}
