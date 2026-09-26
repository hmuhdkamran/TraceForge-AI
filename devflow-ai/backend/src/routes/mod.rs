use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::{
    health::{health, config_handler},
    investigations::{
        create_investigation, list_investigations, get_investigation, get_investigation_events,
        run_baseline, get_baseline, get_bob_investigation_prompt, get_bob_implementation_prompt,
    },
    projects::{list_projects, get_project, list_scenarios},
    artifacts::{sync_artifacts, list_artifacts, list_findings},
    approvals::{get_plan, create_approval},
    verification::{run_verification, get_verification, get_diff},
    reports::{get_report, get_report_html, get_report_json},
    metrics::{get_metrics, record_manual_baseline},
};
use crate::state::SharedState;

pub fn all_routes(state: SharedState) -> Router {
    Router::new()
        // System
        .route("/api/v1/health", get(health))
        .route("/api/v1/config", get(config_handler))
        // Projects
        .route("/api/v1/projects", get(list_projects))
        .route("/api/v1/projects/:id", get(get_project))
        .route("/api/v1/projects/:id/scenarios", get(list_scenarios))
        // Investigations
        .route("/api/v1/investigations", post(create_investigation).get(list_investigations))
        .route("/api/v1/investigations/:id", get(get_investigation))
        .route("/api/v1/investigations/:id/events", get(get_investigation_events))
        // Baseline
        .route("/api/v1/investigations/:id/baseline", post(run_baseline).get(get_baseline))
        // Bob handoff
        .route("/api/v1/investigations/:id/bob/investigation-prompt", get(get_bob_investigation_prompt))
        .route("/api/v1/investigations/:id/bob/implementation-prompt", get(get_bob_implementation_prompt))
        .route("/api/v1/investigations/:id/artifacts/sync", post(sync_artifacts))
        .route("/api/v1/investigations/:id/artifacts", get(list_artifacts))
        .route("/api/v1/investigations/:id/findings", get(list_findings))
        // Approval
        .route("/api/v1/investigations/:id/plan", get(get_plan))
        .route("/api/v1/investigations/:id/approval", post(create_approval))
        // Verification
        .route("/api/v1/investigations/:id/verification", post(run_verification).get(get_verification))
        .route("/api/v1/investigations/:id/diff", get(get_diff))
        // Reports
        .route("/api/v1/investigations/:id/report", get(get_report))
        .route("/api/v1/investigations/:id/report.html", get(get_report_html))
        .route("/api/v1/investigations/:id/report.json", get(get_report_json))
        // Metrics
        .route("/api/v1/investigations/:id/metrics", get(get_metrics))
        .route("/api/v1/investigations/:id/manual-baseline", post(record_manual_baseline))
        .with_state(state)
}
