import { useParams } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { Download, FileText, CheckCircle, ExternalLink } from 'lucide-react';
import {
  getInvestigation,
  getInvestigationEvents,
  runBaseline,
  getReport,
} from '../services/api';
import { StatusBadge } from '../components/shared/StatusBadge';
import { WorkflowStepper } from '../components/investigation/WorkflowStepper';
import { BobWorkspacePanel } from '../components/investigation/BobWorkspacePanel';
import { FindingsPanel } from '../components/investigation/FindingsPanel';
import { ApprovalPanel } from '../components/investigation/ApprovalPanel';
import { VerificationPanel } from '../components/verification/VerificationPanel';
import { EvidenceGraph } from '../components/evidence/EvidenceGraph';
import { MetricsPanel } from '../components/investigation/MetricsPanel';
import { formatDatetime } from '../utils/status';

export function InvestigationDetailPage() {
  const { id } = useParams<{ id: string }>();
  const qc = useQueryClient();
  const [activeTab, setActiveTab] = useState<
    'overview' | 'bob' | 'findings' | 'approval' | 'verification' | 'evidence' | 'metrics' | 'report' | 'events'
  >('overview');

  const { data: investigation, isLoading } = useQuery({
    queryKey: ['investigation', id],
    queryFn: () => getInvestigation(id!),
    refetchInterval: 5_000,
    enabled: !!id,
  });

  const baselineMutation = useMutation({
    mutationFn: () => runBaseline(id!),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['investigation', id] }),
  });

  if (isLoading) {
    return <div className="p-8 text-center text-gray-400">Loading investigation...</div>;
  }

  if (!investigation) {
    return <div className="p-8 text-center text-red-500">Investigation not found.</div>;
  }

  const apiBase = import.meta.env.VITE_API_URL || '/api/v1';

  const tabs = [
    { id: 'overview', label: 'Overview' },
    { id: 'bob', label: 'IBM Bob' },
    { id: 'findings', label: 'Findings' },
    { id: 'approval', label: 'Approval' },
    { id: 'verification', label: 'Verification' },
    { id: 'evidence', label: 'Evidence Graph' },
    { id: 'metrics', label: 'Metrics' },
    { id: 'report', label: 'Report' },
    { id: 'events', label: 'Audit Trail' },
  ] as const;

  return (
    <div className="space-y-5">
      {/* Header */}
      <div className="bg-white rounded-lg border border-gray-200 p-5 shadow-sm">
        <div className="flex items-start justify-between gap-4">
          <div>
            <h1 className="text-xl font-bold text-gray-900">{investigation.title}</h1>
            <p className="text-gray-500 text-sm mt-1">{investigation.description}</p>
            <div className="flex items-center gap-4 mt-3 text-xs text-gray-400">
              <span>ID: <code className="font-mono">{investigation.id.slice(0, 8)}...</code></span>
              <span>Project: <code className="font-mono">{investigation.project_id}</code></span>
              <span>Created: {formatDatetime(investigation.created_at)}</span>
            </div>
          </div>
          <div className="flex flex-col items-end gap-2">
            <StatusBadge status={investigation.status} large />
            <div className="flex items-center gap-2 mt-1">
              <a
                href={`${apiBase}/investigations/${investigation.id}/report.html`}
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-1.5 text-xs text-gray-700 bg-gray-50 hover:bg-gray-100 border border-gray-300 px-2.5 py-1 rounded transition-colors"
                title="View full standalone HTML report"
              >
                <Download size={13} />
                HTML Report
              </a>
              <a
                href={`${apiBase}/investigations/${investigation.id}/report.json`}
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-1.5 text-xs text-gray-700 bg-gray-50 hover:bg-gray-100 border border-gray-300 px-2.5 py-1 rounded transition-colors"
                title="Download JSON report export"
              >
                <Download size={13} />
                JSON
              </a>
            </div>
          </div>
        </div>

        <div className="mt-4">
          <WorkflowStepper status={investigation.status} />
        </div>

        {/* Quick actions */}
        <div className="flex gap-2 mt-4 flex-wrap">
          {['WORKSPACE_READY', 'BASELINE_CAPTURED', 'AWAITING_BOB_INVESTIGATION'].includes(investigation.status) && (
            <button
              onClick={() => baselineMutation.mutate()}
              disabled={baselineMutation.isPending || investigation.status === 'BASELINE_RUNNING'}
              className="text-xs bg-blue-600 text-white px-3 py-1.5 rounded hover:bg-blue-700 disabled:opacity-50"
            >
              {baselineMutation.isPending ? 'Starting...' : 'Run Baseline Tests'}
            </button>
          )}
        </div>
      </div>

      {/* Tabs */}
      <div className="bg-white rounded-lg border border-gray-200 shadow-sm">
        <div className="flex border-b border-gray-100 overflow-x-auto">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`px-5 py-3 text-sm font-medium whitespace-nowrap border-b-2 transition-colors ${
                activeTab === tab.id
                  ? 'border-blue-600 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-900'
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>

        <div className="p-5">
          {activeTab === 'overview' && (
            <OverviewTab investigation={investigation} />
          )}
          {activeTab === 'bob' && (
            <BobWorkspacePanel
              investigation={investigation}
              onSync={() => {
                qc.invalidateQueries({ queryKey: ['investigation', id] });
                qc.invalidateQueries({ queryKey: ['findings', id] });
                qc.invalidateQueries({ queryKey: ['plan', id] });
                qc.invalidateQueries({ queryKey: ['investigation-events', id] });
                qc.invalidateQueries({ queryKey: ['metrics', id] });
                qc.invalidateQueries({ queryKey: ['report', id] });
              }}
            />
          )}
          {activeTab === 'findings' && (
            <FindingsPanel investigationId={id!} />
          )}
          {activeTab === 'approval' && (
            <ApprovalPanel
              investigation={investigation}
              onApproved={() => {
                qc.invalidateQueries({ queryKey: ['investigation', id] });
                qc.invalidateQueries({ queryKey: ['metrics', id] });
              }}
            />
          )}
          {activeTab === 'verification' && (
            <VerificationPanel
              investigation={investigation}
              onVerify={() => {
                qc.invalidateQueries({ queryKey: ['investigation', id] });
                qc.invalidateQueries({ queryKey: ['metrics', id] });
                qc.invalidateQueries({ queryKey: ['report', id] });
              }}
            />
          )}
          {activeTab === 'evidence' && (
            <EvidenceGraph investigationId={id!} />
          )}
          {activeTab === 'metrics' && (
            <MetricsPanel investigation={investigation} />
          )}
          {activeTab === 'report' && (
            <ReportTab investigationId={id!} apiBase={apiBase} />
          )}
          {activeTab === 'events' && (
            <EventsTab investigationId={id!} />
          )}
        </div>
      </div>
    </div>
  );
}

function OverviewTab({ investigation }: { investigation: ReturnType<typeof useQuery>['data'] & object }) {
  return (
    <div className="space-y-4 text-sm">
      <div className="grid grid-cols-2 gap-4">
        <div>
          <div className="text-xs text-gray-500 mb-1">Status</div>
          <StatusBadge status={(investigation as any).status} />
        </div>
        <div>
          <div className="text-xs text-gray-500 mb-1">Workspace ID</div>
          <code className="font-mono text-xs text-gray-700">{(investigation as any).workspace_id}</code>
        </div>
        {(investigation as any).approved_at && (
          <div>
            <div className="text-xs text-gray-500 mb-1">Approved At</div>
            <span className="text-emerald-700">{formatDatetime((investigation as any).approved_at)}</span>
          </div>
        )}
        {(investigation as any).completed_at && (
          <div>
            <div className="text-xs text-gray-500 mb-1">Completed At</div>
            <span className="text-emerald-700">{formatDatetime((investigation as any).completed_at)}</span>
          </div>
        )}
        {(investigation as any).failure_reason && (
          <div className="col-span-2">
            <div className="text-xs text-gray-500 mb-1">Failure Reason</div>
            <div className="text-red-600 bg-red-50 border border-red-200 p-2 rounded text-xs">
              {(investigation as any).failure_reason}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function ReportTab({ investigationId, apiBase }: { investigationId: string; apiBase: string }) {
  const { data: report, isLoading } = useQuery({
    queryKey: ['report', investigationId],
    queryFn: () => getReport(investigationId),
  });

  if (isLoading) {
    return <div className="text-gray-400 text-sm">Loading report details...</div>;
  }

  const r = report as any;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between border-b border-gray-100 pb-3">
        <div>
          <h3 className="font-semibold text-gray-900 text-base">Comprehensive Investigation Report</h3>
          <p className="text-xs text-gray-500">19 verified sections spanning bug reproduction, findings, diffs, and verification</p>
        </div>
        <div className="flex gap-2">
          <a
            href={`${apiBase}/investigations/${investigationId}/report.html`}
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-1.5 text-xs bg-blue-600 hover:bg-blue-700 text-white px-3 py-1.5 rounded font-medium shadow-sm transition-colors"
          >
            <ExternalLink size={14} />
            Open Standalone Report
          </a>
        </div>
      </div>

      {r && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs">
          <div className="border border-gray-200 rounded-lg p-3.5 space-y-2">
            <div className="font-semibold text-gray-800 text-sm">Expected API Contract</div>
            <p className="text-gray-600 leading-relaxed">{r.expected_api_behavior}</p>
          </div>
          <div className="border border-gray-200 rounded-lg p-3.5 space-y-2">
            <div className="font-semibold text-gray-800 text-sm">Observed Defect Behavior</div>
            <p className="text-gray-600 leading-relaxed">{r.observed_behavior}</p>
          </div>
        </div>
      )}

      {/* Remaining Risks & Known Limitations */}
      {r && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs">
          <div className="border border-amber-200 bg-amber-50/50 rounded-lg p-3.5 space-y-2">
            <div className="font-semibold text-amber-900 text-sm">18. Remaining Production Risks</div>
            <ul className="list-disc pl-4 space-y-1.5 text-amber-800">
              {(r.remaining_risks || []).map((risk: string, idx: number) => (
                <li key={idx} className="leading-relaxed">{risk}</li>
              ))}
            </ul>
          </div>
          <div className="border border-gray-200 bg-gray-50/50 rounded-lg p-3.5 space-y-2">
            <div className="font-semibold text-gray-900 text-sm">19. Known Platform Limitations</div>
            <ul className="list-disc pl-4 space-y-1.5 text-gray-700">
              {(r.known_limitations || []).map((lim: string, idx: number) => (
                <li key={idx} className="leading-relaxed">{lim}</li>
              ))}
            </ul>
          </div>
        </div>
      )}

      {/* Embedded Iframe Preview */}
      <div className="border border-gray-200 rounded-lg overflow-hidden shadow-inner">
        <div className="bg-gray-100 px-4 py-2 border-b border-gray-200 text-xs font-medium text-gray-600 flex items-center justify-between">
          <span>Live Standalone HTML Report Preview</span>
          <span className="font-mono text-[11px] text-gray-400">/investigations/{investigationId}/report.html</span>
        </div>
        <iframe
          src={`${apiBase}/investigations/${investigationId}/report.html`}
          title="ContractGuard HTML Report Preview"
          className="w-full h-[600px] border-0 bg-white"
        />
      </div>
    </div>
  );
}

function EventsTab({ investigationId }: { investigationId: string }) {
  const { data: events = [] } = useQuery({
    queryKey: ['events', investigationId],
    queryFn: () => getInvestigationEvents(investigationId),
    refetchInterval: 5_000,
  });

  if (events.length === 0) {
    return <div className="text-gray-400 text-sm">No audit events yet.</div>;
  }

  return (
    <div className="space-y-2">
      {events.map((event) => (
        <div key={event.id} className="flex items-start gap-3 text-sm">
          <div className="text-xs text-gray-400 w-40 shrink-0 pt-0.5">{formatDatetime(event.created_at)}</div>
          <div>
            <span className="font-mono text-xs text-blue-700">{event.event_type}</span>
            <span className="text-gray-400 ml-2 text-xs">by {event.actor_type}</span>
          </div>
        </div>
      ))}
    </div>
  );
}
