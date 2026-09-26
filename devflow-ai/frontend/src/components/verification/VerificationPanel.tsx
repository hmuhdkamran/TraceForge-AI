import { useQuery, useMutation } from '@tanstack/react-query';
import { PlayCircle, CheckCircle, XCircle, Clock } from 'lucide-react';
import { getVerification, runVerification, getDiff } from '../../services/api';
import type { Investigation } from '../../types';
import { formatDuration, formatDatetime } from '../../utils/status';

interface Props {
  investigation: Investigation;
  onVerify: () => void;
}

export function VerificationPanel({ investigation, onVerify }: Props) {
  const { data: verificationData, isLoading } = useQuery({
    queryKey: ['verification', investigation.id],
    queryFn: () => getVerification(investigation.id),
    refetchInterval: investigation.status === 'VERIFICATION_RUNNING' ? 3000 : false,
  });

  const { data: diffData } = useQuery({
    queryKey: ['diff', investigation.id],
    queryFn: () => getDiff(investigation.id),
    enabled: ['READY_FOR_VERIFICATION', 'VERIFICATION_RUNNING', 'VERIFIED', 'VERIFICATION_FAILED', 'COMPLETED'].includes(investigation.status),
  });

  const runMutation = useMutation({
    mutationFn: () => runVerification(investigation.id),
    onSuccess: onVerify,
  });

  const baseline = verificationData?.baseline;
  const verification = verificationData?.verification;

  if (isLoading) return <div className="text-gray-400 text-sm">Loading verification data...</div>;

  return (
    <div className="space-y-5">
      <div className="flex items-start justify-between">
        <div>
          <h3 className="font-semibold text-gray-900 mb-1">Verification</h3>
          <p className="text-sm text-gray-500">Independent test execution before and after the fix.</p>
        </div>
        {investigation.approved_at && !['VERIFIED', 'COMPLETED'].includes(investigation.status) && (
          <button
            onClick={() => runMutation.mutate()}
            disabled={runMutation.isPending || investigation.status === 'VERIFICATION_RUNNING'}
            className="flex items-center gap-2 bg-blue-600 text-white px-4 py-2 rounded-md text-sm font-medium hover:bg-blue-700 disabled:opacity-50"
          >
            <PlayCircle size={16} />
            {runMutation.isPending ? 'Starting...' : 'Run Verification'}
          </button>
        )}
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <ExecutionCard
          title="Baseline (Before Fix)"
          execution={baseline || null}
          type="baseline"
        />
        <ExecutionCard
          title="Verification (After Fix)"
          execution={verification || null}
          type="verification"
        />
      </div>

      {/* Implementation diff */}
      {diffData && (
        <div>
          <div className="text-sm font-medium text-gray-700 mb-2">Implementation Summary</div>
          <pre className="bg-gray-50 border border-gray-200 rounded p-3 text-xs font-mono text-gray-700 whitespace-pre-wrap max-h-48 overflow-y-auto">
            {diffData.implementation_summary}
          </pre>
        </div>
      )}

      {runMutation.error && (
        <div className="text-red-600 text-sm">{String(runMutation.error)}</div>
      )}
    </div>
  );
}

function ExecutionCard({ title, execution, type }: { title: string; execution: any; type: string }) {
  if (!execution) {
    return (
      <div className="border border-gray-200 rounded-lg p-4">
        <div className="text-xs font-medium text-gray-500 mb-2">{title}</div>
        <div className="flex items-center gap-2 text-gray-400 text-sm">
          <Clock size={16} />
          Not yet executed
        </div>
      </div>
    );
  }

  const statusIcon = {
    passed: <CheckCircle size={16} className="text-emerald-500" />,
    failed: <XCircle size={16} className="text-red-500" />,
    running: <Clock size={16} className="text-blue-500 animate-spin" />,
    timeout: <XCircle size={16} className="text-amber-500" />,
    error: <XCircle size={16} className="text-red-500" />,
  }[execution.status] || <Clock size={16} className="text-gray-400" />;

  return (
    <div className={`border rounded-lg p-4 ${
      execution.status === 'passed' ? 'border-emerald-200 bg-emerald-50' :
      execution.status === 'failed' ? 'border-red-200 bg-red-50' :
      'border-gray-200'
    }`}>
      <div className="text-xs font-medium text-gray-600 mb-2">{title}</div>
      <div className="flex items-center gap-2 text-sm font-medium mb-3">
        {statusIcon}
        <span className="capitalize">{execution.status}</span>
        {execution.exit_code !== null && execution.exit_code !== undefined && (
          <span className="text-xs text-gray-400">(exit {execution.exit_code})</span>
        )}
      </div>
      <div className="grid grid-cols-2 gap-2 text-xs text-gray-600">
        <div>Duration: <span className="font-mono">{formatDuration(execution.duration_ms)}</span></div>
        <div>Started: <span>{formatDatetime(execution.started_at)}</span></div>
      </div>
      {execution.summary && (
        <div className="mt-2 text-xs text-gray-700 font-mono bg-white/60 rounded px-2 py-1">
          {execution.summary}
        </div>
      )}
    </div>
  );
}
