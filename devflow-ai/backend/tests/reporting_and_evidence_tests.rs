use axum::extract::{Path, State};
use axum::Json;
use contractguard_lib::config::Config;
use contractguard_lib::handlers::{
    approvals::create_approval,
    artifacts::sync_artifacts,
    metrics::{get_metrics, record_manual_baseline, ManualBaselineRequest},
    reports::{get_report, get_report_html, get_report_json},
};
use contractguard_lib::models::investigation::{ApprovalRequest, CreateInvestigationRequest};
use contractguard_lib::services::investigation::InvestigationService;
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
    let test_dir = std::env::temp_dir().join(format!("contractguard_report_test_{}", test_name));
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
async fn test_get_report_all_19_sections() {
    let db = setup_test_db().await;
    let (config, _test_dir) = test_config("all_19_sections");

    let req = CreateInvestigationRequest {
        title: "UploadLab Batch Ingestion Defect".to_string(),
        description: "Multi-file batch uploads drop subsequent files silently.".to_string(),
        project_id: "uploadlab".to_string(),
        scenario_id: "uploadlab_bug_001_002".to_string(),
        expected_behavior: Some(
            "All files in batch are processed and returned in 200 OK".to_string(),
        ),
        observed_behavior: Some("Only first file processed; subsequent files omitted".to_string()),
        reproduction_steps: Some("Submit multipart batch with 2 files".to_string()),
    };

    let inv = InvestigationService::create(&db, &config, req)
        .await
        .expect("Failed to create investigation");

    let state = Arc::new(AppState::new(db.clone(), config.clone()));

    // Create artifacts on disk and sync them
    let ws_dir = config
        .workspace_root
        .join(&inv.workspace_id)
        .join("bob_artifacts");
    std::fs::create_dir_all(&ws_dir).unwrap();

    let findings_json = serde_json::json!({
        "version": "1.0",
        "investigation_id": inv.id,
        "findings": [
            {
                "id": "FINDING-001",
                "finding_type": "frontend_defect",
                "title": "Frontend FormData Field Name Mismatch",
                "description": "UploadForm.tsx sends 'file' instead of 'files'",
                "severity": "HIGH",
                "confidence": "HIGH",
                "evidence": [
                    {
                        "source_file": "sample_project/broken/frontend/src/components/UploadForm.tsx",
                        "start_line": 23,
                        "end_line": 25,
                        "description": "Form data append key"
                    }
                ]
            },
            {
                "id": "FINDING-002",
                "finding_type": "root_cause",
                "title": "Backend Multipart Stream Truncated Loop",
                "description": "upload.rs early breaks after first multipart item",
                "severity": "CRITICAL",
                "confidence": "HIGH",
                "evidence": [
                    {
                        "source_file": "sample_project/broken/backend/src/routes/upload.rs",
                        "start_line": 48,
                        "end_line": 52,
                        "description": "Conditional if let instead of while let"
                    }
                ]
            }
        ]
    });
    std::fs::write(
        ws_dir.join("findings.json"),
        serde_json::to_string_pretty(&findings_json).unwrap(),
    )
    .unwrap();

    let fix_plan = "# Correction Plan\n1. Change 'file' to 'files' in UploadForm.tsx\n2. Change 'if let' to 'while let' in upload.rs\n";
    std::fs::write(ws_dir.join("fix_plan.md"), fix_plan).unwrap();

    let summary = "# Implementation Summary\nCorrected frontend key and backend loop iteration.\n";
    std::fs::write(ws_dir.join("implementation_summary.md"), summary).unwrap();

    let changed_files = serde_json::json!({
        "version": "1.0",
        "investigation_id": inv.id,
        "files": [
            {
                "relative_path": "sample_project/broken/backend/src/routes/upload.rs",
                "change_type": "modify",
                "rationale": "Iterate all fields with while-let"
            },
            {
                "relative_path": "sample_project/broken/frontend/src/components/UploadForm.tsx",
                "change_type": "modify",
                "rationale": "Change field name to 'files'"
            }
        ]
    });
    std::fs::write(
        ws_dir.join("changed_files.json"),
        serde_json::to_string_pretty(&changed_files).unwrap(),
    )
    .unwrap();

    // Sync artifacts
    let sync_res = sync_artifacts(State(state.clone()), Path(inv.id.clone())).await;
    assert!(sync_res.is_ok(), "Artifact sync must succeed");

    // Approve the plan
    let approval_req = ApprovalRequest {
        decision: "approved".into(),
        approver_id: Some("sec_lead".to_string()),
        comment: Some("Plan approved for execution".to_string()),
    };
    let approval_res = create_approval(
        State(state.clone()),
        Path(inv.id.clone()),
        Json(approval_req),
    )
    .await;
    assert!(approval_res.is_ok(), "Approval must succeed");

    // Query report
    let report_res = get_report(State(state.clone()), Path(inv.id.clone()))
        .await
        .expect("Report fetch failed");
    let report_val = report_res
        .0
        .get("report")
        .expect("Expected report root key")
        .clone();

    // Verify all 19 required sections are present
    assert!(
        report_val.get("investigation_summary").is_some(),
        "Section 1 missing"
    );
    assert!(
        report_val.get("original_bug_report").is_some(),
        "Section 2 missing"
    );
    assert!(
        report_val.get("expected_api_behavior").is_some(),
        "Section 3 missing"
    );
    assert!(
        report_val.get("observed_behavior").is_some(),
        "Section 4 missing"
    );
    assert!(
        report_val.get("baseline_reproduction").is_some(),
        "Section 5 missing"
    );
    assert!(
        report_val.get("documentation_findings").is_some(),
        "Section 6 missing"
    );
    assert!(
        report_val.get("frontend_findings").is_some(),
        "Section 7 missing"
    );
    assert!(
        report_val.get("backend_findings").is_some(),
        "Section 8 missing"
    );
    assert!(
        report_val.get("confirmed_root_causes").is_some(),
        "Section 9 missing"
    );
    assert!(
        report_val.get("approved_correction_plan").is_some(),
        "Section 10 missing"
    );
    assert!(
        report_val.get("implementation_summary").is_some(),
        "Section 11 missing"
    );
    assert!(
        report_val.get("modified_files").is_some(),
        "Section 12 missing"
    );
    assert!(
        report_val.get("actual_code_diff").is_some(),
        "Section 13 missing"
    );
    assert!(
        report_val.get("regression_test_results").is_some(),
        "Section 14 missing"
    );
    assert!(
        report_val.get("independent_verification_status").is_some(),
        "Section 15 missing"
    );
    assert!(
        report_val.get("evidence_graph").is_some(),
        "Section 16 missing"
    );
    assert!(
        report_val.get("productivity_measurements").is_some(),
        "Section 17 missing"
    );
    assert!(
        report_val.get("remaining_risks").is_some(),
        "Section 18 missing"
    );
    assert!(
        report_val.get("known_limitations").is_some(),
        "Section 19 missing"
    );

    // Verify evidence graph nodes & edges
    let graph = report_val.get("evidence_graph").unwrap();
    let nodes = graph
        .get("nodes")
        .and_then(|n| n.as_array())
        .expect("nodes array");
    let edges = graph
        .get("edges")
        .and_then(|e| e.as_array())
        .expect("edges array");
    assert!(!nodes.is_empty(), "Evidence graph must contain nodes");
    assert!(!edges.is_empty(), "Evidence graph must contain edges");

    // Verify modified files entries
    let mod_files = report_val
        .get("modified_files")
        .and_then(|m| m.as_array())
        .unwrap();
    assert_eq!(mod_files.len(), 2, "Expected 2 modified files");

    // Verify approved correction plan
    let plan = report_val.get("approved_correction_plan").unwrap();
    assert_eq!(
        plan.get("approved_by").and_then(|a| a.as_str()),
        Some("sec_lead")
    );
}

#[tokio::test]
async fn test_get_report_html_format_and_escaping() {
    let db = setup_test_db().await;
    let (config, _test_dir) = test_config("html_escaping");

    let req = CreateInvestigationRequest {
        title: "Test <script>alert(1)</script> Report".to_string(),
        description: "Description with <special> & \"characters\"".to_string(),
        project_id: "uploadlab".to_string(),
        scenario_id: "uploadlab_bug_001_002".to_string(),
        expected_behavior: None,
        observed_behavior: None,
        reproduction_steps: None,
    };

    let inv = InvestigationService::create(&db, &config, req)
        .await
        .expect("Failed to create investigation");

    let state = Arc::new(AppState::new(db.clone(), config.clone()));

    let html_res = get_report_html(State(state.clone()), Path(inv.id.clone()))
        .await
        .expect("HTML report failed");
    let html_content = html_res.0;

    // Check basic HTML document structure
    assert!(html_content.contains("<!DOCTYPE html>"), "Missing doctype");
    assert!(
        html_content.contains("<html lang=\"en\">"),
        "Missing html tag"
    );
    assert!(
        html_content.contains("<h1>ContractGuard Investigation Report</h1>"),
        "Missing h1 title"
    );
    assert!(
        html_content.contains("@media print"),
        "Missing print stylesheet"
    );

    // Check origin markers
    assert!(
        html_content.contains("origin-bob"),
        "Missing Bob origin tag"
    );
    assert!(
        html_content.contains("origin-system"),
        "Missing System origin tag"
    );
    assert!(
        html_content.contains("origin-exec"),
        "Missing Executed origin tag"
    );
    assert!(
        html_content.contains("origin-dev"),
        "Missing Dev origin tag"
    );

    // Check all 19 section headers exist in HTML
    for i in 1..=19 {
        let section_header = format!("<h2>{}.", i);
        assert!(
            html_content.contains(&section_header),
            "HTML missing header for section {}",
            i
        );
    }

    // Verify HTML escaping of unsafe input
    assert!(
        !html_content.contains("<script>alert(1)</script>"),
        "Unsafe script tag was not escaped!"
    );
    assert!(
        html_content.contains("&lt;script&gt;alert(1)&lt;/script&gt;"),
        "Script tag must be safely HTML escaped"
    );
    assert!(html_content.contains("&amp;"), "Ampersand must be escaped");
}

#[tokio::test]
async fn test_get_report_json_export() {
    let db = setup_test_db().await;
    let (config, _test_dir) = test_config("json_export");

    let req = CreateInvestigationRequest {
        title: "JSON Export Verification".to_string(),
        description: "Checking complete JSON serialization".to_string(),
        project_id: "uploadlab".to_string(),
        scenario_id: "uploadlab_bug_001_002".to_string(),
        expected_behavior: None,
        observed_behavior: None,
        reproduction_steps: None,
    };

    let inv = InvestigationService::create(&db, &config, req)
        .await
        .expect("Failed to create investigation");

    let state = Arc::new(AppState::new(db.clone(), config.clone()));

    let json_res = get_report_json(State(state.clone()), Path(inv.id.clone()))
        .await
        .expect("JSON report failed");
    let json_val = json_res.0;

    assert_eq!(
        json_val.get("investigation_id").and_then(|v| v.as_str()),
        Some(inv.id.as_str())
    );
    assert!(json_val.get("investigation_summary").is_some());
    assert!(json_val.get("actual_code_diff").is_some());
    assert!(json_val.get("evidence_graph").is_some());
    assert!(json_val.get("remaining_risks").is_some());
    assert!(json_val.get("known_limitations").is_some());
}

#[tokio::test]
async fn test_metrics_calculation_and_manual_baseline_comparison() {
    let db = setup_test_db().await;
    let (config, _test_dir) = test_config("metrics_calc");

    let req = CreateInvestigationRequest {
        title: "Metrics Test Investigation".to_string(),
        description: "Testing productivity metrics and manual comparison".to_string(),
        project_id: "uploadlab".to_string(),
        scenario_id: "uploadlab_bug_001_002".to_string(),
        expected_behavior: None,
        observed_behavior: None,
        reproduction_steps: None,
    };

    let inv = InvestigationService::create(&db, &config, req)
        .await
        .expect("Failed to create investigation");

    let state = Arc::new(AppState::new(db.clone(), config.clone()));

    // Record a manual developer baseline: 45 active minutes
    let manual_req = ManualBaselineRequest {
        scenario_id: "uploadlab_bug_001_002".to_string(),
        start_time: "2026-09-27T10:00:00Z".to_string(),
        finish_time: Some("2026-09-27T10:45:00Z".to_string()),
        active_minutes: Some(45),
        completed_steps: Some(vec![
            "manual_investigation".to_string(),
            "code_edit".to_string(),
            "manual_test_run".to_string(),
        ]),
        final_test_status: Some("passed".to_string()),
    };

    let rec_res =
        record_manual_baseline(State(state.clone()), Path(inv.id.clone()), Json(manual_req)).await;
    assert!(rec_res.is_ok(), "Record manual baseline must succeed");

    // Fetch computed metrics
    let metrics_res = get_metrics(State(state.clone()), Path(inv.id.clone()))
        .await
        .expect("Get metrics failed");
    let metrics_obj = metrics_res.0.get("metrics").expect("metrics object");

    assert_eq!(
        metrics_obj.get("investigation_id").and_then(|v| v.as_str()),
        Some(inv.id.as_str())
    );

    // Verify manual baseline comparison
    let manual_comp = metrics_obj
        .get("manual_comparison")
        .expect("manual_comparison must exist");
    assert!(!manual_comp.is_null(), "manual_comparison must not be null");

    let manual_mins = manual_comp
        .get("manual_active_minutes")
        .and_then(|v| v.as_i64())
        .unwrap();
    let bob_mins = manual_comp
        .get("bob_active_minutes")
        .and_then(|v| v.as_i64())
        .unwrap();
    let saved_mins = manual_comp
        .get("time_saved_minutes")
        .and_then(|v| v.as_i64())
        .unwrap();
    let pct_red = manual_comp
        .get("percentage_time_reduction")
        .and_then(|v| v.as_i64())
        .unwrap();

    assert_eq!(manual_mins, 45, "Manual active minutes should be 45");
    assert!(
        bob_mins <= manual_mins,
        "Bob active minutes should be <= manual"
    );
    assert_eq!(
        saved_mins,
        manual_mins - bob_mins,
        "Saved minutes formula: manual - bob"
    );
    assert!(
        pct_red >= 0 && pct_red <= 100,
        "Percentage reduction should be in [0, 100]"
    );
}
