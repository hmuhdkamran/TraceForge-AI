import { useState } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import { ShieldCheck, ShieldX, AlertCircle } from 'lucide-react';
import { getPlan, createApproval } from '../../services/api';
import type { Investigation } from '../../types';
import { formatDatetime } from '../../utils/status';

interface Props {
  investigation: Investigation;
  onApproved: () => void;
}

export function ApprovalPanel({ investigation, onApproved }: Props) {
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [comment, setComment] = useState('');
  const [pendingDecision, setPendingDecision] = useState<'approved' | 'rejected' | null>(null);

  const { data: planData, isLoading } = useQuery({
    queryKey: ['plan', investigation.id],
    queryFn: () => getPlan(investigation.id),
  });

  const approvalMutation = useMutation({
    mutationFn: ({ decision, comment }: { decision: 'approved' | 'rejected'; comment?: string }) =>
      createApproval(investigation.id, decision, comment),
    onSuccess: () => {
      setConfirmOpen(false);
      onApproved();
    },
  });

  const handleDecision = (decision: 'approved' | 'rejected') => {
    setPendingDecision(decision);
    setConfirmOpen(true);
  };

  const handleConfirm = () => {
    if (pendingDecision) {
      approvalMutation.mutate({ decision: pendingDecision, comment: comment || undefined });
    }
  };

  if (isLoading) return <div className="text-gray-400 text-sm">Loading plan...</div>;

  const alreadyApproved = planData?.approval?.decision === 'approved';

  return (
    <div className="space-y-5">
      <div>
        <h3 className="font-semibold text-gray-900 mb-1">Fix Approval</h3>
        <p className="text-sm text-gray-500">
          Review the proposed fix plan and approve or reject it. Approved plans are immutable.
        </p>
      </div>

      {/* Plan */}
      <div>
        <div className="flex items-center justify-between mb-2">
          <div className="text-xs font-medium text-gray-600">Proposed Fix Plan</div>
          {planData?.plan_hash && (
            <code className="text-xs font-mono text-gray-400">
              SHA: {planData.plan_hash.slice(0, 12)}...
            </code>
          )}
        </div>
        <pre className="bg-gray-50 border border-gray-200 rounded p-4 text-xs text-gray-700 whitespace-pre-wrap max-h-64 overflow-y-auto font-mono">
          {planData?.fix_plan || 'No fix plan available yet. Sync Bob investigation artifacts first.'}
        </pre>
      </div>

      {/* Approval status */}
      {planData?.approval && (
        <div className={`flex items-center gap-3 p-3 rounded-md border text-sm ${
          planData.approval.decision === 'approved'
            ? 'bg-emerald-50 border-emerald-200 text-emerald-700'
            : 'bg-red-50 border-red-200 text-red-700'
        }`}>
          {planData.approval.decision === 'approved'
            ? <ShieldCheck size={18} />
            : <ShieldX size={18} />
          }
          <div>
            <div className="font-medium capitalize">{planData.approval.decision}</div>
            <div className="text-xs">
              {formatDatetime(planData.approval.approved_at)}
              {planData.approval.comment && ` — ${planData.approval.comment}`}
            </div>
          </div>
        </div>
      )}

      {/* Actions */}
      {!alreadyApproved && planData?.fix_plan && (
        <div className="flex gap-3">
          <button
            onClick={() => handleDecision('approved')}
            className="flex items-center gap-2 bg-emerald-600 text-white px-4 py-2 rounded-md text-sm font-medium hover:bg-emerald-700"
          >
            <ShieldCheck size={16} />
            Approve Fix
          </button>
          <button
            onClick={() => handleDecision('rejected')}
            className="flex items-center gap-2 bg-red-600 text-white px-4 py-2 rounded-md text-sm font-medium hover:bg-red-700"
          >
            <ShieldX size={16} />
            Reject / Request Changes
          </button>
        </div>
      )}

      {!planData?.fix_plan && (
        <div className="flex items-center gap-2 text-amber-700 bg-amber-50 border border-amber-200 rounded-md p-3 text-sm">
          <AlertCircle size={16} />
          No fix plan available. Sync Bob artifacts from the IBM Bob tab first.
        </div>
      )}

      {/* Confirmation dialog */}
      {confirmOpen && (
        <div className="fixed inset-0 bg-black/40 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl p-6 max-w-md w-full mx-4">
            <h3 className="font-semibold text-gray-900 mb-2">
              {pendingDecision === 'approved' ? 'Approve Fix Plan?' : 'Reject Fix Plan?'}
            </h3>
            <p className="text-sm text-gray-500 mb-4">
              {pendingDecision === 'approved'
                ? 'This will lock the plan. Implementation can proceed after approval.'
                : 'This will reject the plan. The investigation will need to be re-investigated.'}
            </p>
            <div className="mb-4">
              <label className="block text-sm font-medium text-gray-700 mb-1">Comment (optional)</label>
              <textarea
                value={comment}
                onChange={(e) => setComment(e.target.value)}
                rows={3}
                className="w-full border border-gray-300 rounded px-3 py-2 text-sm"
                placeholder="Add a comment..."
              />
            </div>
            <div className="flex gap-3 justify-end">
              <button
                onClick={() => setConfirmOpen(false)}
                className="px-4 py-2 text-sm text-gray-600 hover:bg-gray-100 rounded"
              >
                Cancel
              </button>
              <button
                onClick={handleConfirm}
                disabled={approvalMutation.isPending}
                className={`px-4 py-2 text-sm text-white rounded font-medium ${
                  pendingDecision === 'approved'
                    ? 'bg-emerald-600 hover:bg-emerald-700'
                    : 'bg-red-600 hover:bg-red-700'
                } disabled:opacity-50`}
              >
                {approvalMutation.isPending ? 'Processing...' : 'Confirm'}
              </button>
            </div>
            {approvalMutation.error && (
              <div className="mt-2 text-red-600 text-sm">{String(approvalMutation.error)}</div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
