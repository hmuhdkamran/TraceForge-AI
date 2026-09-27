use std::path::PathBuf;
use std::sync::Arc;
use axum::extract::{Path, State};
use sqlx::sqlite::SqlitePoolOptions;
use contractguard_lib::config::Config;
use contractguard_lib::models::investigation::CreateInvestigationRequest;
use contractguard_lib::repositories::{InvestigationRepo, FindingRepo, EvidenceRepo, AuditRepo};
use contractguard_lib::services::investigation::InvestigationService;
use contractguard_lib::state::AppState;
use contractguard_lib::handlers::artifacts::{sync_artifacts, list_artifacts, list_findings};

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
    let test_dir = std::env::temp_dir().join(format!("contractguard_artifact_test_{}", test_name));
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
    };

    (config, test_dir)
}

#[tokio::test]
async fn test_successful_artifact_sync_and_findings() {
    let db = setup_test_db().await;
    let (config, test_dir) = test_config("success_sync");

    let req = CreateInvestigationRequest {
        title: "Multi-file Upload Investigation".into(),
        description: "Investigate frontend and backend contract drift".into(),
        project_id: "uploadlab".into(),
        scenario_id: "multi-file-upload-failure".into(),
        expected_behavior: Some("All files uploaded in 'files' field".into()),
        observed_behavior: Some("Only 1 file uploaded, backend drops others".into()),
        reproduction_steps: Some("Select 3 files and click upload".into()),
    };

    let inv = InvestigationService::create(&db, &config, req)
        .await
        .expect("Failed to create investigation");

    let ws_path = config.workspace_root.join(&inv.workspace_id);
    let bob_artifacts_dir = ws_path.join("bob_artifacts");
    std::fs::create_dir_all(&bob_artifacts_dir).expect("Failed to create bob_artifacts dir");

    // 1. Write diagnosis.json
    let diagnosis_json = serde_json::json!({
        "schema_version": 1,
        "investigation_id": inv.id,
        "project_id": "uploadlab",
        "summary": "Root cause confirmed: Frontend sends 'file' multipart field, backend only processes 1st file in loop.",
        "requirements": [
            "POST /api/upload accepts multipart form-data with field name 'files'",
            "All provided files must be processed and returned in uploaded array"
        ],
        "findings": [
            {
                "id": "FINDING-001",
                "finding_type": "contract_mismatch",
                "title": "Frontend sends 'file' multipart field instead of 'files'",
                "description": "UploadForm.tsx appends each file using key 'file' rather than 'files', violating OpenAPI specification.",
                "expected_behavior": "FormData contains entries with key 'files'",
                "observed_behavior": "FormData contains entries with key 'file'",
                "confidence": "high",
                "source_references": [
                    {
                        "relative_path": "broken/frontend/src/components/UploadForm.tsx",
                        "start_line": 35,
                        "end_line": 42,
                        "description": "formData.append('file', f) loop"
                    }
                ]
            },
            {
                "id": "FINDING-002",
                "finding_type": "backend_defect",
                "title": "Backend terminates multipart processing after first file",
                "description": "main.rs contains an early return inside the while let Some(field) loop after saving the first file.",
                "expected_behavior": "Backend processes all fields in the multipart stream",
                "observed_behavior": "Backend returns response after the first iteration",
                "confidence": "high",
                "source_references": [
                    {
                        "relative_path": "broken/backend/src/main.rs",
                        "start_line": 70,
                        "end_line": 85,
                        "description": "break statement after first file"
                    }
                ]
            }
        ],
        "root_causes": [
            {
                "id": "RC-001",
                "title": "Multipart loop premature termination",
                "description": "Early break statement prevents iteration over subsequent files.",
                "finding_ids": ["FINDING-002"]
            }
        ],
        "evidence": [
            {
                "finding_id": "FINDING-002",
                "evidence_type": "source_reference",
                "source_file": "broken/backend/src/main.rs",
                "start_line": 70,
                "end_line": 85,
                "description": "Loop termination in handler"
            }
        ],
        "affected_files": [
            "broken/frontend/src/components/UploadForm.tsx",
            "broken/backend/src/main.rs"
        ],
        "test_references": [
            "test_upload_two_valid_files_bug002",
            "test_upload_three_valid_files_bug002"
        ],
        "risks": [
            "Clients relying on single-file behavior might need backward compatibility"
        ]
    });

    std::fs::write(
        bob_artifacts_dir.join("diagnosis.json"),
        serde_json::to_string_pretty(&diagnosis_json).unwrap(),
    ).expect("Failed to write diagnosis.json");

    // 2. Write investigation_summary.md
    std::fs::write(
        bob_artifacts_dir.join("investigation_summary.md"),
        "# Investigation Summary\n\nIdentified two independent contract defects:\n1. Frontend field name mismatch\n2. Backend loop break\n",
    ).expect("Failed to write investigation_summary.md");

    // 3. Write fix_plan.md
    std::fs::write(
        bob_artifacts_dir.join("fix_plan.md"),
        "# Fix Plan\n\n1. Modify UploadForm.tsx to use 'files'\n2. Remove premature break in main.rs\n3. Run cargo test and npm test\n",
    ).expect("Failed to write fix_plan.md");

    // 4. Write changed_files.json
    let changed_files_json = serde_json::json!({
        "schema_version": 1,
        "investigation_id": inv.id,
        "plan_hash": "sha256:placeholder",
        "files": [
            {
                "relative_path": "broken/frontend/src/components/UploadForm.tsx",
                "change_type": "modified",
                "description": "Fix multipart field name to 'files'"
            },
            {
                "relative_path": "broken/backend/src/main.rs",
                "change_type": "modified",
                "description": "Process all files without early break"
            }
        ]
    });
    std::fs::write(
        bob_artifacts_dir.join("changed_files.json"),
        serde_json::to_string_pretty(&changed_files_json).unwrap(),
    ).expect("Failed to write changed_files.json");

    // Perform Sync via Handler
    let shared_state = Arc::new(AppState::new(db.clone(), config.clone()));
    let res = sync_artifacts(State(shared_state.clone()), Path(inv.id.clone()))
        .await
        .expect("sync_artifacts failed");

    let val = res.0;
    assert_eq!(val["artifacts_imported"], 4);
    assert_eq!(val["findings_created"], 2);
    let errors = val["errors"].as_array().unwrap();
    assert!(errors.is_empty(), "Expected no sync errors: {:?}", errors);

    // Verify DB findings
    let findings = FindingRepo::list(&db, &inv.id).await.unwrap();
    assert_eq!(findings.len(), 2);
    assert!(findings.iter().any(|f| f.finding_type == "contract_mismatch"));
    assert!(findings.iter().any(|f| f.finding_type == "backend_defect"));

    // Verify DB evidence & excerpt extraction
    let evidence = EvidenceRepo::list_for_investigation(&db, &inv.id).await.unwrap();
    assert!(!evidence.is_empty(), "Evidence should have been imported");
    // At least one evidence item should have content_excerpt populated
    assert!(
        evidence.iter().any(|e| e.content_excerpt.is_some()),
        "Evidence should contain extracted content excerpt from workspace files"
    );

    // Verify investigation status advanced to AWAITING_APPROVAL
    let updated_inv = InvestigationRepo::get_by_id(&db, &inv.id).await.unwrap().unwrap();
    assert_eq!(updated_inv.status, "AWAITING_APPROVAL");

    // Verify audit events
    let audit_events = AuditRepo::list_for_investigation(&db, &inv.id).await.unwrap();
    assert!(audit_events.iter().any(|e| e.event_type == "artifacts.synced"));
    assert!(audit_events.iter().any(|e| e.event_type == "status.investigation_imported"));
    assert!(audit_events.iter().any(|e| e.event_type == "status.awaiting_approval"));

    // Test List Artifacts endpoint
    let list_res = list_artifacts(State(shared_state.clone()), Path(inv.id.clone()))
        .await
        .expect("list_artifacts failed");
    let artifacts_val = list_res.0;
    let artifacts_arr = artifacts_val["artifacts"].as_array().unwrap();
    assert_eq!(artifacts_arr.len(), 4);

    // Test List Findings endpoint
    let list_f_res = list_findings(State(shared_state.clone()), Path(inv.id.clone()))
        .await
        .expect("list_findings failed");
    let findings_val = list_f_res.0;
    let f_arr = findings_val["findings"].as_array().unwrap();
    assert_eq!(f_arr.len(), 2);

    // Test Idempotent Re-sync (Syncing again must NOT duplicate findings)
    let re_res = sync_artifacts(State(shared_state.clone()), Path(inv.id.clone()))
        .await
        .expect("re-sync failed");
    assert_eq!(re_res.0["findings_created"], 2);

    let re_findings = FindingRepo::list(&db, &inv.id).await.unwrap();
    assert_eq!(re_findings.len(), 2, "Idempotent sync must not duplicate findings");

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[tokio::test]
async fn test_artifact_sync_rejects_investigation_id_mismatch() {
    let db = setup_test_db().await;
    let (config, test_dir) = test_config("mismatch_id");

    let req = CreateInvestigationRequest {
        title: "Mismatch ID Test".into(),
        description: "Checking rejection of mismatched ID".into(),
        project_id: "uploadlab".into(),
        scenario_id: "multi-file-upload-failure".into(),
        expected_behavior: None,
        observed_behavior: None,
        reproduction_steps: None,
    };

    let inv = InvestigationService::create(&db, &config, req).await.unwrap();
    let ws_path = config.workspace_root.join(&inv.workspace_id);
    let bob_artifacts_dir = ws_path.join("bob_artifacts");
    std::fs::create_dir_all(&bob_artifacts_dir).unwrap();

    let diagnosis_json = serde_json::json!({
        "schema_version": 1,
        "investigation_id": "wrong-different-id",
        "project_id": "uploadlab",
        "summary": "Should be rejected",
        "findings": [
            {
                "id": "FINDING-001",
                "finding_type": "contract_mismatch",
                "title": "Dummy finding",
                "description": "Dummy"
            }
        ]
    });

    std::fs::write(
        bob_artifacts_dir.join("diagnosis.json"),
        serde_json::to_string_pretty(&diagnosis_json).unwrap(),
    ).unwrap();

    let shared_state = Arc::new(AppState::new(db.clone(), config.clone()));
    let res = sync_artifacts(State(shared_state), Path(inv.id.clone())).await.unwrap();
    let val = res.0;

    assert_eq!(val["findings_created"], 0);
    let errors = val["errors"].as_array().unwrap();
    assert!(!errors.is_empty(), "Should report mismatch error");
    assert!(errors[0].as_str().unwrap().contains("investigation_id mismatch"));

    // Verify investigation status was NOT modified
    let check_inv = InvestigationRepo::get_by_id(&db, &inv.id).await.unwrap().unwrap();
    assert_eq!(check_inv.status, "WORKSPACE_READY");

    let findings = FindingRepo::list(&db, &inv.id).await.unwrap();
    assert!(findings.is_empty());

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[tokio::test]
async fn test_artifact_sync_rejects_path_traversal() {
    let db = setup_test_db().await;
    let (config, test_dir) = test_config("path_traversal");

    let req = CreateInvestigationRequest {
        title: "Path Traversal Test".into(),
        description: "Checking rejection of malicious paths".into(),
        project_id: "uploadlab".into(),
        scenario_id: "multi-file-upload-failure".into(),
        expected_behavior: None,
        observed_behavior: None,
        reproduction_steps: None,
    };

    let inv = InvestigationService::create(&db, &config, req).await.unwrap();
    let ws_path = config.workspace_root.join(&inv.workspace_id);
    let bob_artifacts_dir = ws_path.join("bob_artifacts");
    std::fs::create_dir_all(&bob_artifacts_dir).unwrap();

    let diagnosis_json = serde_json::json!({
        "schema_version": 1,
        "investigation_id": inv.id,
        "project_id": "uploadlab",
        "summary": "Should be rejected due to path traversal",
        "findings": [
            {
                "id": "FINDING-001",
                "finding_type": "contract_mismatch",
                "title": "Exploit attempt",
                "description": "Trying to read host files",
                "source_references": [
                    {
                        "relative_path": "../../etc/shadow",
                        "start_line": 1,
                        "end_line": 10
                    }
                ]
            }
        ]
    });

    std::fs::write(
        bob_artifacts_dir.join("diagnosis.json"),
        serde_json::to_string_pretty(&diagnosis_json).unwrap(),
    ).unwrap();

    let shared_state = Arc::new(AppState::new(db.clone(), config.clone()));
    let res = sync_artifacts(State(shared_state), Path(inv.id.clone())).await.unwrap();
    let val = res.0;

    assert_eq!(val["findings_created"], 0);
    let errors = val["errors"].as_array().unwrap();
    assert!(!errors.is_empty(), "Should report path traversal error");
    assert!(errors[0].as_str().unwrap().contains("Path traversal rejected"));

    // Verify investigation status was NOT modified
    let check_inv = InvestigationRepo::get_by_id(&db, &inv.id).await.unwrap().unwrap();
    assert_eq!(check_inv.status, "WORKSPACE_READY");

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[tokio::test]
async fn test_artifact_sync_rejects_duplicate_finding_ids() {
    let db = setup_test_db().await;
    let (config, test_dir) = test_config("duplicate_ids");

    let req = CreateInvestigationRequest {
        title: "Duplicate ID Test".into(),
        description: "Checking rejection of duplicate finding IDs".into(),
        project_id: "uploadlab".into(),
        scenario_id: "multi-file-upload-failure".into(),
        expected_behavior: None,
        observed_behavior: None,
        reproduction_steps: None,
    };

    let inv = InvestigationService::create(&db, &config, req).await.unwrap();
    let ws_path = config.workspace_root.join(&inv.workspace_id);
    let bob_artifacts_dir = ws_path.join("bob_artifacts");
    std::fs::create_dir_all(&bob_artifacts_dir).unwrap();

    let diagnosis_json = serde_json::json!({
        "schema_version": 1,
        "investigation_id": inv.id,
        "project_id": "uploadlab",
        "summary": "Should be rejected due to duplicate IDs",
        "findings": [
            {
                "id": "FINDING-001",
                "finding_type": "contract_mismatch",
                "title": "Finding 1",
                "description": "First finding"
            },
            {
                "id": "FINDING-001",
                "finding_type": "backend_defect",
                "title": "Finding 2 with duplicate ID",
                "description": "Second finding"
            }
        ]
    });

    std::fs::write(
        bob_artifacts_dir.join("diagnosis.json"),
        serde_json::to_string_pretty(&diagnosis_json).unwrap(),
    ).unwrap();

    let shared_state = Arc::new(AppState::new(db.clone(), config.clone()));
    let res = sync_artifacts(State(shared_state), Path(inv.id.clone())).await.unwrap();
    let val = res.0;

    assert_eq!(val["findings_created"], 0);
    let errors = val["errors"].as_array().unwrap();
    assert!(!errors.is_empty(), "Should report duplicate ID error");
    assert!(errors[0].as_str().unwrap().contains("Duplicate finding IDs"));

    let _ = std::fs::remove_dir_all(&test_dir);
}
