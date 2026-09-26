-- ContractGuard database schema
-- Migration 001: initial schema

CREATE TABLE IF NOT EXISTS investigations (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    project_id TEXT NOT NULL,
    scenario_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'CREATED',
    workspace_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    approved_at TEXT,
    completed_at TEXT,
    failure_reason TEXT
);

CREATE INDEX IF NOT EXISTS idx_investigations_status ON investigations(status);
CREATE INDEX IF NOT EXISTS idx_investigations_project ON investigations(project_id);
CREATE INDEX IF NOT EXISTS idx_investigations_created ON investigations(created_at);

CREATE TABLE IF NOT EXISTS investigation_artifacts (
    id TEXT PRIMARY KEY,
    investigation_id TEXT NOT NULL REFERENCES investigations(id) ON DELETE CASCADE,
    artifact_type TEXT NOT NULL,
    schema_version INTEGER NOT NULL DEFAULT 1,
    relative_path TEXT NOT NULL,
    content_sha256 TEXT NOT NULL,
    created_at TEXT NOT NULL,
    imported_at TEXT NOT NULL,
    validation_status TEXT NOT NULL DEFAULT 'PENDING'
);

CREATE INDEX IF NOT EXISTS idx_artifacts_investigation ON investigation_artifacts(investigation_id);

CREATE TABLE IF NOT EXISTS findings (
    id TEXT PRIMARY KEY,
    investigation_id TEXT NOT NULL REFERENCES investigations(id) ON DELETE CASCADE,
    finding_type TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    expected_behavior TEXT NOT NULL DEFAULT '',
    observed_behavior TEXT NOT NULL DEFAULT '',
    confidence TEXT NOT NULL DEFAULT 'medium',
    verification_status TEXT NOT NULL DEFAULT 'unverified',
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_findings_investigation ON findings(investigation_id);

CREATE TABLE IF NOT EXISTS evidence (
    id TEXT PRIMARY KEY,
    investigation_id TEXT NOT NULL REFERENCES investigations(id) ON DELETE CASCADE,
    finding_id TEXT NOT NULL REFERENCES findings(id) ON DELETE CASCADE,
    evidence_type TEXT NOT NULL,
    source_file TEXT NOT NULL DEFAULT '',
    start_line INTEGER,
    end_line INTEGER,
    content_excerpt TEXT,
    test_id TEXT,
    description TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_evidence_investigation ON evidence(investigation_id);
CREATE INDEX IF NOT EXISTS idx_evidence_finding ON evidence(finding_id);

CREATE TABLE IF NOT EXISTS approvals (
    id TEXT PRIMARY KEY,
    investigation_id TEXT NOT NULL REFERENCES investigations(id) ON DELETE CASCADE,
    plan_hash TEXT NOT NULL,
    decision TEXT NOT NULL,
    approved_at TEXT NOT NULL,
    approver_id TEXT,
    comment TEXT
);

CREATE INDEX IF NOT EXISTS idx_approvals_investigation ON approvals(investigation_id);

CREATE TABLE IF NOT EXISTS test_executions (
    id TEXT PRIMARY KEY,
    investigation_id TEXT NOT NULL REFERENCES investigations(id) ON DELETE CASCADE,
    execution_type TEXT NOT NULL,
    command_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    exit_code INTEGER,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    duration_ms INTEGER,
    stdout_path TEXT,
    stderr_path TEXT,
    summary TEXT
);

CREATE INDEX IF NOT EXISTS idx_executions_investigation ON test_executions(investigation_id);
CREATE INDEX IF NOT EXISTS idx_executions_type ON test_executions(investigation_id, execution_type);

CREATE TABLE IF NOT EXISTS audit_events (
    id TEXT PRIMARY KEY,
    investigation_id TEXT REFERENCES investigations(id) ON DELETE SET NULL,
    event_type TEXT NOT NULL,
    actor_type TEXT NOT NULL,
    actor_id TEXT,
    details_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_audit_investigation ON audit_events(investigation_id);
CREATE INDEX IF NOT EXISTS idx_audit_created ON audit_events(created_at);

CREATE TABLE IF NOT EXISTS manual_baselines (
    id TEXT PRIMARY KEY,
    investigation_id TEXT REFERENCES investigations(id),
    scenario_id TEXT NOT NULL,
    start_time TEXT NOT NULL,
    finish_time TEXT,
    active_minutes INTEGER,
    completed_steps TEXT,
    final_test_status TEXT,
    created_at TEXT NOT NULL
);
