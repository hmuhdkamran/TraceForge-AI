use sqlx::SqlitePool;
use serde::{Serialize, Deserialize};

use crate::errors::ApiResult;
use crate::models::investigation::Investigation;
use crate::repositories::{InvestigationRepo, FindingRepo, EvidenceRepo, TestExecutionRepo, AuditRepo};

#[derive(Debug, Serialize, Deserialize)]
pub struct InvestigationReport {
    pub investigation_id: String,
    pub title: String,
    pub status: String,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub findings: Vec<serde_json::Value>,
    pub evidence: Vec<serde_json::Value>,
    pub baseline_execution: Option<serde_json::Value>,
    pub verification_execution: Option<serde_json::Value>,
    pub audit_events: Vec<serde_json::Value>,
}

pub struct ReportingService;

impl ReportingService {
    pub async fn generate(db: &SqlitePool, investigation: &Investigation) -> ApiResult<InvestigationReport> {
        let findings = FindingRepo::list(db, &investigation.id).await?;
        let evidence = EvidenceRepo::list_for_investigation(db, &investigation.id).await?;
        let audit_events = AuditRepo::list_for_investigation(db, &investigation.id).await?;

        let baseline = TestExecutionRepo::get_latest(db, &investigation.id, "baseline").await?;
        let verification = TestExecutionRepo::get_latest(db, &investigation.id, "verification").await?;

        Ok(InvestigationReport {
            investigation_id: investigation.id.clone(),
            title: investigation.title.clone(),
            status: investigation.status.clone(),
            created_at: investigation.created_at.clone(),
            completed_at: investigation.completed_at.clone(),
            findings: findings.iter().map(|f| serde_json::to_value(f).unwrap_or_default()).collect(),
            evidence: evidence.iter().map(|e| serde_json::to_value(e).unwrap_or_default()).collect(),
            baseline_execution: baseline.map(|e| serde_json::to_value(e).unwrap_or_default()),
            verification_execution: verification.map(|e| serde_json::to_value(e).unwrap_or_default()),
            audit_events: audit_events.iter().map(|e| serde_json::to_value(e).unwrap_or_default()).collect(),
        })
    }

    pub fn render_html(report: &InvestigationReport) -> String {
        let findings_html = report.findings.iter().map(|f| {
            let title = f["title"].as_str().unwrap_or("Unknown");
            let desc = html_escape(f["description"].as_str().unwrap_or(""));
            let etype = html_escape(f["finding_type"].as_str().unwrap_or(""));
            let confidence = html_escape(f["confidence"].as_str().unwrap_or(""));
            format!(
                r#"<div class="finding">
  <h3 class="finding-title">{title}</h3>
  <span class="badge">{etype}</span> <span class="badge badge-confidence">{confidence}</span>
  <p>{desc}</p>
</div>"#,
            )
        }).collect::<Vec<_>>().join("\n");

        let status_class = match report.status.as_str() {
            "COMPLETED" | "VERIFIED" => "status-success",
            "FAILED" | "VERIFICATION_FAILED" => "status-error",
            _ => "status-pending",
        };

        format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>ContractGuard Report — {title}</title>
<style>
body {{ font-family: -apple-system, 'Segoe UI', system-ui, sans-serif; margin: 0; padding: 2rem; background: #f7f8fa; color: #1f2328; }}
.container {{ max-width: 900px; margin: 0 auto; background: white; border-radius: 8px; padding: 2rem; border: 1px solid #e5e7eb; }}
h1 {{ color: #1e3a5f; }}
h2 {{ color: #1e3a5f; border-bottom: 1px solid #e5e7eb; padding-bottom: 0.5rem; }}
.status {{ display: inline-block; padding: 4px 12px; border-radius: 4px; font-weight: 600; }}
.status-success {{ background: #d1fae5; color: #065f46; }}
.status-error {{ background: #fee2e2; color: #991b1b; }}
.status-pending {{ background: #fef3c7; color: #92400e; }}
.badge {{ display: inline-block; padding: 2px 8px; border-radius: 3px; background: #e5e7eb; font-size: 0.85em; margin-right: 4px; }}
.badge-confidence {{ background: #dbeafe; color: #1e40af; }}
.finding {{ border: 1px solid #e5e7eb; border-radius: 6px; padding: 1rem; margin: 0.5rem 0; }}
.finding-title {{ margin: 0 0 0.5rem; }}
pre {{ background: #f7f8fa; padding: 1rem; border-radius: 4px; overflow-x: auto; white-space: pre-wrap; }}
table {{ width: 100%; border-collapse: collapse; }}
td, th {{ padding: 0.5rem; border: 1px solid #e5e7eb; text-align: left; }}
th {{ background: #f7f8fa; }}
footer {{ text-align: center; margin-top: 2rem; color: #57606a; font-size: 12px; border-top: 1px solid #e5e7eb; padding-top: 1rem; }}
</style>
</head>
<body>
<div class="container">
<h1>ContractGuard Investigation Report</h1>
<table>
<tr><th>Investigation ID</th><td><code>{investigation_id}</code></td></tr>
<tr><th>Title</th><td>{title}</td></tr>
<tr><th>Status</th><td><span class="status {status_class}">{status}</span></td></tr>
<tr><th>Created</th><td>{created_at}</td></tr>
</table>

<h2>Findings ({finding_count})</h2>
{findings_html}

<h2>Audit Trail</h2>
<table>
<tr><th>Time</th><th>Event</th><th>Actor</th></tr>
{audit_rows}
</table>

<footer>Generated by ContractGuard &mdash; TraceForge AI</footer>
</div>
</body>
</html>"#,
            title = html_escape(&report.title),
            investigation_id = html_escape(&report.investigation_id),
            status = html_escape(&report.status),
            status_class = status_class,
            created_at = html_escape(&report.created_at),
            finding_count = report.findings.len(),
            findings_html = findings_html,
            audit_rows = report.audit_events.iter().map(|e| {
                format!("<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                    html_escape(e["created_at"].as_str().unwrap_or("")),
                    html_escape(e["event_type"].as_str().unwrap_or("")),
                    html_escape(e["actor_type"].as_str().unwrap_or(""))
                )
            }).collect::<Vec<_>>().join("\n"),
        )
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
