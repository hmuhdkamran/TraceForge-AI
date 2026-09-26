import { useParams } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import {
  getInvestigation,
  getInvestigationEvents,
  runBaseline,
  getBobInvestigationPrompt,
  syncArtifacts,
  listFindings,
  getPlan,
  createApproval,
  runVerification,
  getVerification,
  getDiff,
  getMetrics,
} from '../services/api';
import { StatusBadge } from '../components/shared/StatusBadge';
import { WorkflowStepper } from '../components/investigation/WorkflowStepper';
import { BobWorkspacePanel } from '../components/investigation/BobWorkspacePanel';
import { FindingsPanel } from '../components/investigation/FindingsPanel';
import { ApprovalPanel } from '../components/investigation/ApprovalPanel';
import { VerificationPanel } from '../components/verification/VerificationPanel';
import { EvidenceGraph } from '../components/evidence/EvidenceGraph';
import { formatDatetime } from '../utils/status';

export function InvestigationDetailPage() {
  const { id } = useParams<{ id: string }>();
  const qc = useQueryClient();
  const [activeTab, setActiveTab] = useState<'overview' | 'bob' | 'findings' | 'approval' | 'verification' | 'evidence' | 'events'>('overview');

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

  const tabs = [
    { id: 'overview', label: 'Overview' },
    { id: 'bob', label: 'IBM Bob' },
    { id: 'findings', label: 'Findings' },
    { id: 'approval', label: 'Approval' },
    { id: 'verification', label: 'Verification' },
    { id: 'evidence', label: 'Evidence Graph' },
    { id: 'events', label: 'Audit Trail' },
  ] as const;

  return (
    <div className="space-y-5">
      {/* Header */}
      <div className="bg-white rounded-lg border border-gray-200 p-5">
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
          <StatusBadge status={investigation.status} large />
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
      <div className="bg-white rounded-lg border border-gray-200">
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
            <BobWorkspacePanel investigation={investigation} onSync={() => qc.invalidateQueries({ queryKey: ['investigation', id] })} />
          )}
          {activeTab === 'findings' && (
            <FindingsPanel investigationId={id!} />
          )}
          {activeTab === 'approval' && (
            <ApprovalPanel investigation={investigation} onApproved={() => qc.invalidateQueries({ queryKey: ['investigation', id] })} />
          )}
          {activeTab === 'verification' && (
            <VerificationPanel investigation={investigation} onVerify={() => qc.invalidateQueries({ queryKey: ['investigation', id] })} />
          )}
          {activeTab === 'evidence' && (
            <EvidenceGraph investigationId={id!} />
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
