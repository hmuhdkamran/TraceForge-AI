use sqlx::SqlitePool;
use serde::{Serialize, Deserialize};

use crate::errors::ApiResult;
use crate::models::investigation::Investigation;
use crate::repositories::{FindingRepo, TestExecutionRepo};

#[derive(Debug, Serialize, Deserialize)]
pub struct InvestigationMetrics {
    pub investigation_id: String,
    pub total_duration_ms: Option<i64>,
    pub baseline_duration_ms: Option<i64>,
    pub verification_duration_ms: Option<i64>,
    pub baseline_tests_failed: i64,
    pub verification_tests_passed: i64,
    pub confirmed_root_causes: i64,
    pub modified_files: i64,
    pub regression_tests_added: i64,
    pub status: String,
}

pub struct MetricsService;

impl MetricsService {
    pub async fn compute(db: &SqlitePool, investigation: &Investigation) -> ApiResult<InvestigationMetrics> {
        let findings = FindingRepo::list(db, &investigation.id).await?;
        let root_causes = findings.iter()
            .filter(|f| f.finding_type == "root_cause")
            .count() as i64;

        let baseline_execs = TestExecutionRepo::list_for_investigation(
            db,
            &investigation.id,
            Some("baseline"),
        ).await?;

        let verification_execs = TestExecutionRepo::list_for_investigation(
            db,
            &investigation.id,
            Some("verification"),
        ).await?;

        let baseline_duration = baseline_execs.first().and_then(|e| e.duration_ms);
        let verification_duration = verification_execs.first().and_then(|e| e.duration_ms);

        // Calculate total duration from created_at to updated_at
        let total_duration = {
            let created = chrono::DateTime::parse_from_rfc3339(&investigation.created_at).ok();
            let updated = chrono::DateTime::parse_from_rfc3339(&investigation.updated_at).ok();
            match (created, updated) {
                (Some(c), Some(u)) => Some((u.timestamp_millis() - c.timestamp_millis()).max(0)),
                _ => None,
            }
        };

        Ok(InvestigationMetrics {
            investigation_id: investigation.id.clone(),
            total_duration_ms: total_duration,
            baseline_duration_ms: baseline_duration,
            verification_duration_ms: verification_duration,
            baseline_tests_failed: 0, // updated after parsing results
            verification_tests_passed: 0,
            confirmed_root_causes: root_causes,
            modified_files: 0,
            regression_tests_added: 0,
            status: investigation.status.clone(),
        })
    }
}
