import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { ChevronDown, ChevronRight, AlertTriangle } from 'lucide-react';
import { listFindings } from '../../services/api';
import type { Finding, Evidence } from '../../types';

interface Props {
  investigationId: string;
}

export function FindingsPanel({ investigationId }: Props) {
  const { data, isLoading } = useQuery({
    queryKey: ['findings', investigationId],
    queryFn: () => listFindings(investigationId),
  });

  const findings = data?.findings ?? [];
  const evidence = data?.evidence ?? [];

  if (isLoading) return <div className="text-gray-400 text-sm">Loading findings...</div>;

  if (findings.length === 0) {
    return (
      <div className="text-center py-8">
        <AlertTriangle size={32} className="mx-auto text-amber-400 mb-2" />
        <p className="text-gray-500 text-sm">No findings yet. Sync Bob investigation artifacts to populate findings.</p>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {findings.map((finding) => {
        const findingEvidence = evidence.filter((e) => e.finding_id === finding.id);
        return <FindingCard key={finding.id} finding={finding} evidence={findingEvidence} />;
      })}
    </div>
  );
}

function FindingCard({ finding, evidence }: { finding: Finding; evidence: Evidence[] }) {
  const [expanded, setExpanded] = useState(false);

  const confidenceColor = {
    high: 'text-red-600 bg-red-50',
    medium: 'text-amber-600 bg-amber-50',
    low: 'text-gray-600 bg-gray-50',
  }[finding.confidence] || 'text-gray-600 bg-gray-50';

  const typeLabel: Record<string, string> = {
    contract_mismatch: 'Contract Mismatch',
    backend_defect: 'Backend Defect',
    frontend_defect: 'Frontend Defect',
    root_cause: 'Root Cause',
    documentation: 'Documentation',
  };

  return (
    <div className="border border-gray-200 rounded-lg overflow-hidden">
      <button
        onClick={() => setExpanded((e) => !e)}
        className="w-full flex items-start gap-3 p-4 text-left hover:bg-gray-50"
      >
        <div className="mt-0.5 text-gray-400 shrink-0">
          {expanded ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 flex-wrap">
            <span className="font-medium text-gray-900 text-sm">{finding.title}</span>
            <span className={`text-xs px-2 py-0.5 rounded font-medium ${confidenceColor}`}>
              {finding.confidence} confidence
            </span>
            <span className="text-xs px-2 py-0.5 rounded bg-blue-50 text-blue-700 font-medium">
              {typeLabel[finding.finding_type] || finding.finding_type}
            </span>
          </div>
          <p className="text-xs text-gray-500 mt-1 line-clamp-2">{finding.description}</p>
        </div>
      </button>

      {expanded && (
        <div className="border-t border-gray-100 p-4 space-y-3 bg-gray-50 text-sm">
          {finding.expected_behavior && (
            <div>
              <div className="text-xs font-medium text-gray-500 mb-1">Expected Behavior</div>
              <div className="text-gray-700">{finding.expected_behavior}</div>
            </div>
          )}
          {finding.observed_behavior && (
            <div>
              <div className="text-xs font-medium text-gray-500 mb-1">Observed Behavior</div>
              <div className="text-gray-700 text-red-700">{finding.observed_behavior}</div>
            </div>
          )}

          {evidence.length > 0 && (
            <div>
              <div className="text-xs font-medium text-gray-500 mb-2">Source Evidence ({evidence.length})</div>
              <div className="space-y-2">
                {evidence.map((ev) => (
                  <div key={ev.id} className="bg-white border border-gray-200 rounded p-3">
                    {ev.source_file && (
                      <div className="flex items-center gap-2 mb-1">
                        <code className="text-xs font-mono text-blue-700">{ev.source_file}</code>
                        {ev.start_line && (
                          <span className="text-xs text-gray-400">
                            L{ev.start_line}{ev.end_line ? `–${ev.end_line}` : ''}
                          </span>
                        )}
                      </div>
                    )}
                    {ev.content_excerpt && (
                      <pre className="text-xs font-mono text-gray-600 bg-gray-50 p-2 rounded overflow-x-auto whitespace-pre-wrap">
                        {ev.content_excerpt}
                      </pre>
                    )}
                    <div className="text-xs text-gray-500 mt-1">{ev.description}</div>
                  </div>
                ))}
              </div>
            </div>
          )}

          <div className="text-xs text-gray-400">
            Verification: <span className="font-medium">{finding.verification_status}</span>
          </div>
        </div>
      )}
    </div>
  );
}
