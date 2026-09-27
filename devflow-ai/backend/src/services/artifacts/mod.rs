use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::errors::{AppError, ApiResult};
use crate::repositories::{InvestigationRepo, FindingRepo, EvidenceRepo, AuditRepo};
use crate::security::path_guard::PathGuard;
use crate::services::workspace::WorkspaceService;

const MAX_ARTIFACT_SIZE: usize = 2 * 1024 * 1024; // 2 MB

pub struct ArtifactService;

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosisArtifact {
    pub schema_version: u32,
    pub investigation_id: String,
    pub project_id: String,
    pub summary: String,
    #[serde(default)]
    pub requirements: Vec<String>,
    pub findings: Vec<DiagnosisFinding>,
    #[serde(default)]
    pub root_causes: Vec<RootCause>,
    #[serde(default)]
    pub evidence: Vec<DiagnosisEvidence>,
    #[serde(default)]
    pub affected_files: Vec<String>,
    #[serde(default)]
    pub test_references: Vec<String>,
    #[serde(default)]
    pub risks: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosisFinding {
    pub id: String,
    pub finding_type: String,
    pub title: String,
    pub description: String,
    pub expected_behavior: Option<String>,
    pub observed_behavior: Option<String>,
    pub confidence: Option<String>,
    pub source_references: Option<Vec<SourceReference>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RootCause {
    pub id: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub finding_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosisEvidence {
    pub finding_id: String,
    pub evidence_type: String,
    pub source_file: Option<String>,
    pub start_line: Option<i64>,
    pub end_line: Option<i64>,
    pub content_excerpt: Option<String>,
    pub test_id: Option<String>,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceReference {
    pub relative_path: String,
    pub start_line: Option<i64>,
    pub end_line: Option<i64>,
    pub description: Option<String>,
}

pub struct SyncResult {
    pub artifacts_imported: usize,
    pub findings_created: usize,
    pub errors: Vec<String>,
}

impl ArtifactService {
    pub async fn sync_artifacts(
        db: &SqlitePool,
        config: &Config,
        investigation_id: &str,
        workspace_id: &str,
    ) -> ApiResult<SyncResult> {
        let mut result = SyncResult {
            artifacts_imported: 0,
            findings_created: 0,
            errors: vec![],
        };

        // Check for diagnosis.json
        let diagnosis_path = format!("bob_artifacts/diagnosis.json");
        match WorkspaceService::read_artifact(config, workspace_id, &diagnosis_path).await {
            Ok(content) => {
                if content.len() > MAX_ARTIFACT_SIZE {
                    result.errors.push("diagnosis.json exceeds size limit".into());
                } else {
                    match Self::import_diagnosis(db, config, investigation_id, workspace_id, &content).await {
                        Ok(count) => {
                            result.artifacts_imported += 1;
                            result.findings_created += count;
                        }
                        Err(e) => {
                            result.errors.push(format!("diagnosis.json: {}", e));
                        }
                    }
                }
            }
            Err(AppError::NotFound(_)) => {
                // Not yet present — not an error
            }
            Err(e) => {
                result.errors.push(format!("Failed to read diagnosis.json: {}", e));
            }
        }

        // Import plain markdown artifacts
        for rel_path in &[
            "bob_artifacts/investigation_summary.md",
            "bob_artifacts/fix_plan.md",
            "bob_artifacts/implementation_summary.md",
            "bob_artifacts/review.md",
        ] {
            match WorkspaceService::read_artifact(config, workspace_id, rel_path).await {
                Ok(content) => {
                    let sha = Self::sha256(&content);
                    let artifact_type = rel_path
                        .trim_start_matches("bob_artifacts/")
                        .trim_end_matches(".md")
                        .replace('_', "-");

                    match InvestigationRepo::upsert_artifact(
                        db,
                        investigation_id,
                        &artifact_type,
                        rel_path,
                        &sha,
                        "valid",
                    ).await {
                        Ok(_) => result.artifacts_imported += 1,
                        Err(e) => result.errors.push(format!("{}: {}", rel_path, e)),
                    }
                }
                Err(AppError::NotFound(_)) => {}
                Err(e) => result.errors.push(format!("Failed to read {}: {}", rel_path, e)),
            }
        }

        // Check for changed_files.json
        let changed_files_path = "bob_artifacts/changed_files.json";
        if let Ok(content) = WorkspaceService::read_artifact(config, workspace_id, changed_files_path).await {
            let sha = Self::sha256(&content);
            let _ = InvestigationRepo::upsert_artifact(
                db,
                investigation_id,
                "changed-files",
                changed_files_path,
                &sha,
                "valid",
            ).await;
            result.artifacts_imported += 1;
        }

        AuditRepo::record(
            db,
            Some(investigation_id),
            "artifacts.synced",
            "developer",
            None,
            serde_json::json!({
                "imported": result.artifacts_imported,
                "findings": result.findings_created,
                "errors": result.errors.len()
            }),
        ).await?;

        Ok(result)
    }

    async fn import_diagnosis(
        db: &SqlitePool,
        config: &Config,
        investigation_id: &str,
        workspace_id: &str,
        content: &str,
    ) -> ApiResult<usize> {
        let diagnosis: DiagnosisArtifact = serde_json::from_str(content)
            .map_err(|e| AppError::ArtifactValidation(format!("diagnosis.json parse error: {}", e)))?;

        if diagnosis.schema_version < 1 {
            return Err(AppError::ArtifactValidation("schema_version must be >= 1".into()));
        }

        if diagnosis.investigation_id.is_empty() {
            return Err(AppError::ArtifactValidation("investigation_id is required".into()));
        }

        if diagnosis.investigation_id != investigation_id {
            return Err(AppError::ArtifactValidation(format!(
                "investigation_id mismatch in diagnosis.json (expected {}, got {})",
                investigation_id, diagnosis.investigation_id
            )));
        }

        // Validate finding IDs are unique
        let ids: std::collections::HashSet<&str> = diagnosis.findings.iter().map(|f| f.id.as_str()).collect();
        if ids.len() != diagnosis.findings.len() {
            return Err(AppError::ArtifactValidation("Duplicate finding IDs in diagnosis.json".into()));
        }

        // Validate paths and line ranges in source_references
        for f in &diagnosis.findings {
            if let Some(refs) = &f.source_references {
                for r in refs {
                    if PathGuard::validate_workspace_path(config, workspace_id, &r.relative_path).is_err() {
                        return Err(AppError::ArtifactValidation(format!(
                            "Path traversal rejected in source_reference: {}", r.relative_path
                        )));
                    }
                    if let (Some(s), Some(e)) = (r.start_line, r.end_line) {
                        if s < 1 || e < s {
                            return Err(AppError::ArtifactValidation(format!(
                                "Invalid line range in source_reference {}: {}-{}",
                                r.relative_path, s, e
                            )));
                        }
                    }
                }
            }
        }

        // Validate paths and line ranges in evidence
        for e in &diagnosis.evidence {
            if let Some(ref path) = e.source_file {
                if PathGuard::validate_workspace_path(config, workspace_id, path).is_err() {
                    return Err(AppError::ArtifactValidation(format!(
                        "Path traversal rejected in evidence source_file: {}", path
                    )));
                }
            }
            if let (Some(s), Some(e)) = (e.start_line, e.end_line) {
                if s < 1 || e < s {
                    return Err(AppError::ArtifactValidation(format!(
                        "Invalid line range in evidence: {}-{}", s, e
                    )));
                }
            }
        }

        let sha = Self::sha256(content);

        // Upsert the artifact record
        InvestigationRepo::upsert_artifact(
            db,
            investigation_id,
            "diagnosis",
            "bob_artifacts/diagnosis.json",
            &sha,
            "valid",
        ).await?;

        // Delete existing findings and evidence to allow idempotent re-import
        EvidenceRepo::delete_for_investigation(db, investigation_id).await?;
        FindingRepo::delete_for_investigation(db, investigation_id).await?;

        let mut finding_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        let mut count = 0;

        for f in &diagnosis.findings {
            let finding = FindingRepo::create(
                db,
                investigation_id,
                &f.finding_type,
                &f.title,
                &f.description,
                f.expected_behavior.as_deref().unwrap_or(""),
                f.observed_behavior.as_deref().unwrap_or(""),
                f.confidence.as_deref().unwrap_or("medium"),
            ).await?;

            finding_map.insert(f.id.clone(), finding.id.clone());
            count += 1;

            // Import source references as evidence with excerpt extraction
            if let Some(refs) = &f.source_references {
                for r in refs {
                    let excerpt = Self::extract_excerpt(config, workspace_id, &r.relative_path, r.start_line, r.end_line);

                    EvidenceRepo::create(
                        db,
                        investigation_id,
                        &finding.id,
                        "source_reference",
                        &r.relative_path,
                        r.start_line,
                        r.end_line,
                        excerpt.as_deref(),
                        None,
                        r.description.as_deref().unwrap_or("Source reference"),
                    ).await?;
                }
            }
        }

        // Import additional evidence mapped to corresponding findings
        for e in &diagnosis.evidence {
            if let Some(finding_uuid) = finding_map.get(&e.finding_id) {
                let excerpt = if e.content_excerpt.is_some() {
                    e.content_excerpt.clone()
                } else if let Some(ref path) = e.source_file {
                    Self::extract_excerpt(config, workspace_id, path, e.start_line, e.end_line)
                } else {
                    None
                };

                EvidenceRepo::create(
                    db,
                    investigation_id,
                    finding_uuid,
                    &e.evidence_type,
                    e.source_file.as_deref().unwrap_or(""),
                    e.start_line,
                    e.end_line,
                    excerpt.as_deref(),
                    e.test_id.as_deref(),
                    &e.description,
                ).await?;
            }
        }

        Ok(count)
    }

    fn extract_excerpt(
        config: &Config,
        workspace_id: &str,
        relative_path: &str,
        start_line: Option<i64>,
        end_line: Option<i64>,
    ) -> Option<String> {
        let full_path = match PathGuard::validate_workspace_path(config, workspace_id, relative_path) {
            Ok(p) => p,
            Err(_) => return None,
        };

        if full_path.exists() && full_path.is_file() {
            if let Ok(file_str) = std::fs::read_to_string(&full_path) {
                let lines: Vec<&str> = file_str.lines().collect();
                if let (Some(s), Some(e)) = (start_line, end_line) {
                    let s_idx = (s as usize).saturating_sub(1);
                    let e_idx = (e as usize).min(lines.len());
                    if s_idx < lines.len() && s_idx < e_idx {
                        return Some(lines[s_idx..e_idx].join("\n"));
                    }
                }
            }
        }
        None
    }

    pub fn sha256(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }
}
