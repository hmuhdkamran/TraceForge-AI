import React from 'react';
import { useQuery } from '@tanstack/react-query';
import { listFindings } from '../../services/api';
import type { Finding, Evidence, EvidenceGraphNode, EvidenceGraphEdge } from '../../types';

interface Props {
  investigationId: string;
}

const NODE_COLORS: Record<string, string> = {
  requirement: '#1e3a5f',
  frontend: '#2563eb',
  backend: '#7c3aed',
  failing_test: '#dc2626',
  root_cause: '#d97706',
  fix: '#059669',
  passing_test: '#0d9488',
};

const NODE_LABELS: Record<string, string> = {
  requirement: 'Requirement',
  frontend: 'Frontend',
  backend: 'Backend',
  failing_test: 'Failing Test',
  root_cause: 'Root Cause',
  fix: 'Fix',
  passing_test: 'Passing Test',
};

export function EvidenceGraph({ investigationId }: Props) {
  const [selected, setSelected] = React.useState<string | null>(null);

  const { data, isLoading } = useQuery({
    queryKey: ['findings', investigationId],
    queryFn: () => listFindings(investigationId),
  });

  const findings = data?.findings ?? [];
  const evidence = data?.evidence ?? [];

  if (isLoading) return <div className="text-gray-400 text-sm">Loading evidence graph...</div>;

  if (findings.length === 0) {
    return (
      <div className="text-center py-8">
        <div className="text-gray-400 text-sm">No evidence yet. Sync Bob artifacts to build the evidence graph.</div>
      </div>
    );
  }

  // Build graph nodes from findings
  const nodes: EvidenceGraphNode[] = findings.map((f) => ({
    id: f.id,
    type: typeFromFinding(f),
    label: f.title,
    data: f,
  }));

  // Build edges: connect findings that share evidence
  const edges: EvidenceGraphEdge[] = [];
  const findingIds = findings.map((f) => f.id);
  for (let i = 0; i < findingIds.length - 1; i++) {
    edges.push({ from: findingIds[i], to: findingIds[i + 1], label: 'leads to' });
  }

  const selectedFinding = findings.find((f) => f.id === selected);
  const selectedEvidence = evidence.filter((e) => e.finding_id === selected);

  return (
    <div className="space-y-4">
      <div>
        <h3 className="font-semibold text-gray-900 mb-1">Contract Evidence Graph</h3>
        <p className="text-sm text-gray-500">
          Traceable relationships between requirements, findings, root causes and fixes.
        </p>
      </div>

      {/* Legend */}
      <div className="flex flex-wrap gap-3">
        {Object.entries(NODE_LABELS).map(([type, label]) => (
          <div key={type} className="flex items-center gap-1.5 text-xs">
            <div className="w-3 h-3 rounded-full" style={{ backgroundColor: NODE_COLORS[type] }} />
            <span className="text-gray-600">{label}</span>
          </div>
        ))}
      </div>

      {/* Graph visualization — connected timeline */}
      <div className="bg-gray-50 border border-gray-200 rounded-lg p-5 overflow-x-auto">
        <div className="flex items-start gap-4">
          {nodes.map((node, idx) => (
            <div key={node.id} className="flex items-center gap-2 shrink-0">
              <button
                onClick={() => setSelected(node.id === selected ? null : node.id)}
                className={`flex flex-col items-center gap-1.5 p-3 rounded-lg border-2 transition-all cursor-pointer max-w-32 ${
                  node.id === selected
                    ? 'border-blue-500 shadow-md bg-white'
                    : 'border-transparent hover:border-gray-300 bg-white hover:shadow-sm'
                }`}
              >
                <div
                  className="w-10 h-10 rounded-full flex items-center justify-center text-white text-xs font-bold"
                  style={{ backgroundColor: NODE_COLORS[node.type] || '#6b7280' }}
                >
                  {nodeIcon(node.type)}
                </div>
                <div className="text-xs font-medium text-gray-700 text-center leading-tight line-clamp-2">
                  {node.label}
                </div>
                <div
                  className="text-xs px-1.5 py-0.5 rounded"
                  style={{ color: NODE_COLORS[node.type], backgroundColor: `${NODE_COLORS[node.type]}20` }}
                >
                  {NODE_LABELS[node.type] || node.type}
                </div>
              </button>
              {idx < nodes.length - 1 && (
                <div className="text-gray-300 text-lg mt-1">→</div>
              )}
            </div>
          ))}
        </div>
      </div>

      {/* Selected node detail */}
      {selectedFinding && (
        <div className="bg-white border border-gray-200 rounded-lg p-4 space-y-3">
          <div className="font-medium text-gray-900">{selectedFinding.title}</div>
          <p className="text-sm text-gray-600">{selectedFinding.description}</p>

          {selectedFinding.expected_behavior && (
            <div>
              <div className="text-xs text-gray-500 mb-1">Expected</div>
              <div className="text-xs text-gray-700">{selectedFinding.expected_behavior}</div>
            </div>
          )}
          {selectedFinding.observed_behavior && (
            <div>
              <div className="text-xs text-gray-500 mb-1">Observed</div>
              <div className="text-xs text-red-700">{selectedFinding.observed_behavior}</div>
            </div>
          )}

          {selectedEvidence.length > 0 && (
            <div>
              <div className="text-xs text-gray-500 mb-2">Evidence ({selectedEvidence.length})</div>
              {selectedEvidence.map((ev) => (
                <div key={ev.id} className="bg-gray-50 rounded p-2 text-xs mb-2">
                  {ev.source_file && (
                    <code className="text-blue-700 font-mono">{ev.source_file}</code>
                  )}
                  {ev.start_line && <span className="text-gray-400 ml-2">L{ev.start_line}{ev.end_line ? `–${ev.end_line}` : ''}</span>}
                  <div className="text-gray-600 mt-1">{ev.description}</div>
                </div>
              ))}
            </div>
          )}

          {selectedEvidence.length === 0 && (
            <div className="text-xs text-amber-600">
              No source evidence stored for this finding. Bob should have provided source references.
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function typeFromFinding(f: Finding): EvidenceGraphNode['type'] {
  switch (f.finding_type) {
    case 'contract_mismatch':
    case 'frontend_defect': return 'frontend';
    case 'backend_defect': return 'backend';
    case 'root_cause': return 'root_cause';
    case 'documentation': return 'requirement';
    default: return 'frontend';
  }
}

function nodeIcon(type: string): string {
  switch (type) {
    case 'requirement': return 'R';
    case 'frontend': return 'F';
    case 'backend': return 'B';
    case 'failing_test': return '✗';
    case 'root_cause': return '!';
    case 'fix': return '✓';
    case 'passing_test': return '✓';
    default: return '?';
  }
}

