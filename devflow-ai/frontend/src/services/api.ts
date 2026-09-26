import axios from 'axios';
import type {
  Investigation,
  CreateInvestigationRequest,
  InvestigationArtifact,
  Finding,
  Evidence,
  Approval,
  TestExecution,
  AuditEvent,
  Project,
  Scenario,
  InvestigationMetrics,
} from '../types';

const BASE_URL = import.meta.env.VITE_API_URL || '/api/v1';

const client = axios.create({ baseURL: BASE_URL });

// --- System ---
export async function getHealth() {
  const res = await client.get('/health');
  return res.data;
}

export async function getConfig() {
  const res = await client.get('/config');
  return res.data;
}

// --- Projects ---
export async function getProjects(): Promise<Project[]> {
  const res = await client.get('/projects');
  return res.data.projects;
}

export async function getProject(id: string): Promise<Project> {
  const res = await client.get(`/projects/${id}`);
  return res.data.project;
}

export async function getScenarios(projectId: string): Promise<Scenario[]> {
  const res = await client.get(`/projects/${projectId}/scenarios`);
  return res.data.scenarios;
}

// --- Investigations ---
export async function createInvestigation(req: CreateInvestigationRequest): Promise<Investigation> {
  const res = await client.post('/investigations', req);
  return res.data.investigation;
}

export async function listInvestigations(limit = 20, offset = 0): Promise<{ investigations: Investigation[]; total: number }> {
  const res = await client.get('/investigations', { params: { limit, offset } });
  return res.data;
}

export async function getInvestigation(id: string): Promise<Investigation> {
  const res = await client.get(`/investigations/${id}`);
  return res.data.investigation;
}

export async function getInvestigationEvents(id: string): Promise<AuditEvent[]> {
  const res = await client.get(`/investigations/${id}/events`);
  return res.data.events;
}

// --- Baseline ---
export async function runBaseline(id: string): Promise<void> {
  await client.post(`/investigations/${id}/baseline`);
}

export async function getBaseline(id: string): Promise<TestExecution[]> {
  const res = await client.get(`/investigations/${id}/baseline`);
  return res.data.baseline_executions;
}

// --- Bob ---
export async function getBobInvestigationPrompt(id: string): Promise<{ prompt: string; workspace_path: string; required_artifacts: string[] }> {
  const res = await client.get(`/investigations/${id}/bob/investigation-prompt`);
  return res.data;
}

export async function getBobImplementationPrompt(id: string): Promise<{ prompt: string; plan_hash: string }> {
  const res = await client.get(`/investigations/${id}/bob/implementation-prompt`);
  return res.data;
}

export async function syncArtifacts(id: string): Promise<{ artifacts_imported: number; findings_created: number; errors: string[] }> {
  const res = await client.post(`/investigations/${id}/artifacts/sync`);
  return res.data;
}

export async function listArtifacts(id: string): Promise<InvestigationArtifact[]> {
  const res = await client.get(`/investigations/${id}/artifacts`);
  return res.data.artifacts;
}

export async function listFindings(id: string): Promise<{ findings: Finding[]; evidence: Evidence[] }> {
  const res = await client.get(`/investigations/${id}/findings`);
  return res.data;
}

// --- Plan & Approval ---
export async function getPlan(id: string): Promise<{ fix_plan: string; plan_hash: string; approval: Approval | null; investigation_status: string }> {
  const res = await client.get(`/investigations/${id}/plan`);
  return res.data;
}

export async function createApproval(id: string, decision: 'approved' | 'rejected', comment?: string): Promise<Approval> {
  const res = await client.post(`/investigations/${id}/approval`, { decision, comment });
  return res.data.approval;
}

// --- Verification ---
export async function runVerification(id: string): Promise<void> {
  await client.post(`/investigations/${id}/verification`);
}

export async function getVerification(id: string): Promise<{ baseline: TestExecution | null; verification: TestExecution | null }> {
  const res = await client.get(`/investigations/${id}/verification`);
  return res.data;
}

export async function getDiff(id: string): Promise<{ implementation_summary: string; changed_files: unknown }> {
  const res = await client.get(`/investigations/${id}/diff`);
  return res.data;
}

// --- Reports ---
export async function getReport(id: string): Promise<unknown> {
  const res = await client.get(`/investigations/${id}/report`);
  return res.data.report;
}

export async function getReportJson(id: string): Promise<unknown> {
  const res = await client.get(`/investigations/${id}/report.json`);
  return res.data;
}

// --- Metrics ---
export async function getMetrics(id: string): Promise<InvestigationMetrics> {
  const res = await client.get(`/investigations/${id}/metrics`);
  return res.data.metrics;
}

export async function recordManualBaseline(id: string, data: {
  scenario_id: string;
  start_time: string;
  finish_time?: string;
  active_minutes?: number;
  completed_steps?: string[];
  final_test_status?: string;
}): Promise<void> {
  await client.post(`/investigations/${id}/manual-baseline`, data);
}
