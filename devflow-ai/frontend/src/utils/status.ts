import type { InvestigationStatus } from '../types';

export function statusLabel(status: InvestigationStatus | string): string {
  const labels: Record<string, string> = {
    CREATED: 'Created',
    WORKSPACE_READY: 'Workspace Ready',
    BASELINE_RUNNING: 'Running Baseline',
    BASELINE_CAPTURED: 'Baseline Captured',
    AWAITING_BOB_INVESTIGATION: 'Awaiting Investigation',
    INVESTIGATION_IMPORTED: 'Investigation Imported',
    AWAITING_APPROVAL: 'Awaiting Approval',
    APPROVED: 'Approved',
    IMPLEMENTATION_IN_PROGRESS: 'Implementation In Progress',
    READY_FOR_VERIFICATION: 'Ready for Verification',
    VERIFICATION_RUNNING: 'Verifying',
    VERIFIED: 'Verified',
    VERIFICATION_FAILED: 'Verification Failed',
    COMPLETED: 'Completed',
    FAILED: 'Failed',
  };
  return labels[status] || status;
}

export function statusColor(status: InvestigationStatus | string): string {
  if (['COMPLETED', 'VERIFIED'].includes(status)) return 'text-emerald-700 bg-emerald-50 border-emerald-200';
  if (['FAILED', 'VERIFICATION_FAILED'].includes(status)) return 'text-red-700 bg-red-50 border-red-200';
  if (['AWAITING_APPROVAL', 'AWAITING_BOB_INVESTIGATION'].includes(status)) return 'text-amber-700 bg-amber-50 border-amber-200';
  if (['APPROVED', 'VERIFICATION_RUNNING'].includes(status)) return 'text-blue-700 bg-blue-50 border-blue-200';
  return 'text-gray-700 bg-gray-50 border-gray-200';
}

export function formatDuration(ms?: number): string {
  if (!ms) return '—';
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60_000) return `${(ms / 1000).toFixed(1)}s`;
  const mins = Math.floor(ms / 60_000);
  const secs = Math.floor((ms % 60_000) / 1000);
  return `${mins}m ${secs}s`;
}

export function formatDatetime(iso?: string): string {
  if (!iso) return '—';
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

export function workflowSteps(): Array<{ status: InvestigationStatus; label: string; step: number }> {
  return [
    { status: 'CREATED', label: 'Created', step: 1 },
    { status: 'WORKSPACE_READY', label: 'Workspace Ready', step: 2 },
    { status: 'BASELINE_CAPTURED', label: 'Baseline Captured', step: 3 },
    { status: 'AWAITING_BOB_INVESTIGATION', label: 'Bob Investigation', step: 4 },
    { status: 'AWAITING_APPROVAL', label: 'Approval', step: 5 },
    { status: 'APPROVED', label: 'Approved', step: 6 },
    { status: 'VERIFIED', label: 'Verified', step: 7 },
    { status: 'COMPLETED', label: 'Completed', step: 8 },
  ];
}

const statusOrder: Record<string, number> = {
  CREATED: 1,
  WORKSPACE_READY: 2,
  BASELINE_RUNNING: 2,
  BASELINE_CAPTURED: 3,
  AWAITING_BOB_INVESTIGATION: 4,
  INVESTIGATION_IMPORTED: 4,
  AWAITING_APPROVAL: 5,
  APPROVED: 6,
  IMPLEMENTATION_IN_PROGRESS: 6,
  READY_FOR_VERIFICATION: 7,
  VERIFICATION_RUNNING: 7,
  VERIFIED: 7,
  VERIFICATION_FAILED: 7,
  COMPLETED: 8,
  FAILED: 0,
};

export function currentStep(status: string): number {
  return statusOrder[status] || 1;
}
