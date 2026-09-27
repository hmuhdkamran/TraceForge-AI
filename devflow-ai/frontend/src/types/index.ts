export type InvestigationStatus =
  | 'CREATED'
  | 'WORKSPACE_READY'
  | 'BASELINE_RUNNING'
  | 'BASELINE_CAPTURED'
  | 'AWAITING_BOB_INVESTIGATION'
  | 'INVESTIGATION_IMPORTED'
  | 'AWAITING_APPROVAL'
  | 'APPROVED'
  | 'IMPLEMENTATION_IN_PROGRESS'
  | 'READY_FOR_VERIFICATION'
  | 'VERIFICATION_RUNNING'
  | 'VERIFIED'
  | 'VERIFICATION_FAILED'
  | 'COMPLETED'
  | 'FAILED';

export interface Investigation {
  id: string;
  title: string;
  description: string;
  project_id: string;
  scenario_id: string;
  status: InvestigationStatus;
  workspace_id: string;
  created_at: string;
  updated_at: string;
  approved_at?: string;
  completed_at?: string;
  failure_reason?: string;
}

export interface Finding {
  id: string;
  investigation_id: string;
  finding_type: string;
  title: string;
  description: string;
  expected_behavior: string;
  observed_behavior: string;
  confidence: string;
  verification_status: string;
  created_at: string;
}

export interface Evidence {
  id: string;
  investigation_id: string;
  finding_id: string;
  evidence_type: string;
  source_file: string;
  start_line?: number;
  end_line?: number;
  content_excerpt?: string;
  test_id?: string;
  description: string;
}

export interface InvestigationArtifact {
  id: string;
  investigation_id: string;
  artifact_type: string;
  schema_version: number;
  relative_path: string;
  content_sha256: string;
  created_at: string;
  imported_at: string;
  validation_status: string;
}

export interface TestExecution {
  id: string;
  investigation_id: string;
  execution_type: string;
  command_id: string;
  status: string;
  exit_code?: number;
  started_at: string;
  finished_at?: string;
  duration_ms?: number;
  stdout_path?: string;
  stderr_path?: string;
  summary?: string;
}

export interface AuditEvent {
  id: string;
  investigation_id?: string;
  event_type: string;
  actor_type: string;
  actor_id?: string;
  details_json: string;
  created_at: string;
}

export interface Approval {
  id: string;
  investigation_id: string;
  plan_hash: string;
  decision: string;
  approved_at: string;
  approver_id?: string;
  comment?: string;
}

export interface Project {
  id: string;
  name: string;
  description: string;
  path: string;
}

export interface Scenario {
  id: string;
  project_id: string;
  name: string;
  description: string;
  bug_report: string;
}

export interface CreateInvestigationRequest {
  title: string;
  description: string;
  project_id: string;
  scenario_id: string;
  expected_behavior?: string;
  observed_behavior?: string;
  reproduction_steps?: string;
}

export interface ManualBaselineComparison {
  manual_active_minutes: number;
  bob_active_minutes: number;
  time_saved_minutes: number;
  percentage_time_reduction: number;
  test_failures_resolved: number;
  verified_regression_tests: number;
  scenario_id: string;
}

export interface InvestigationMetrics {
  investigation_id: string;
  total_duration_ms?: number;
  baseline_duration_ms?: number;
  verification_duration_ms?: number;
  baseline_tests_failed: number;
  verification_tests_passed: number;
  confirmed_root_causes: number;
  modified_files: number;
  regression_tests_added: number;
  status: string;
  manual_comparison?: ManualBaselineComparison;
}

export interface FileDiff {
  path: string;
  diff: string;
}

export interface DiffResponse {
  implementation_summary: string;
  changed_files: unknown;
  diff: string;
  file_diffs?: FileDiff[];
}

export interface EvidenceGraphNode {
  id: string;
  type: 'requirement' | 'frontend' | 'backend' | 'failing_test' | 'root_cause' | 'fix' | 'passing_test';
  label: string;
  data?: Finding | Evidence | Record<string, unknown>;
  description?: string;
}

export interface EvidenceGraphEdge {
  from: string;
  to: string;
  label?: string;
}

export interface EvidenceGraph {
  nodes: EvidenceGraphNode[];
  edges: EvidenceGraphEdge[];
}
