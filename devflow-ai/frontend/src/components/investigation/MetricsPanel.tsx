import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Clock, TrendingUp, CheckCircle, AlertOctagon, FileCode, PlusCircle, Check } from 'lucide-react';
import { getMetrics, recordManualBaseline } from '../../services/api';
import type { Investigation, InvestigationMetrics } from '../../types';
import { formatDuration } from '../../utils/status';

interface Props {
  investigation: Investigation;
}

export function MetricsPanel({ investigation }: Props) {
  const qc = useQueryClient();
  const [showManualForm, setShowManualForm] = useState(false);
  const [manualMinutes, setManualMinutes] = useState(45);
  const [recordSuccess, setRecordSuccess] = useState(false);

  const { data: metrics, isLoading } = useQuery<InvestigationMetrics>({
    queryKey: ['metrics', investigation.id],
    queryFn: () => getMetrics(investigation.id),
    refetchInterval: 5000,
  });

  const recordMutation = useMutation({
    mutationFn: (activeMinutes: number) => {
      const now = new Date();
      const startTime = new Date(now.getTime() - activeMinutes * 60000).toISOString();
      return recordManualBaseline(investigation.id, {
        scenario_id: investigation.scenario_id,
        start_time: startTime,
        finish_time: now.toISOString(),
        active_minutes: activeMinutes,
        completed_steps: [
          'Manual reproduction in browser',
          'Inspect frontend multipart form payload',
          'Diagnose backend stream termination',
          'Apply patch',
          'Re-run test suite',
        ],
        final_test_status: 'passed',
      });
    },
    onSuccess: () => {
      setRecordSuccess(true);
      setShowManualForm(false);
      qc.invalidateQueries({ queryKey: ['metrics', investigation.id] });
      setTimeout(() => setRecordSuccess(false), 4000);
    },
  });

  if (isLoading) {
    return <div className="text-gray-400 text-sm">Loading productivity metrics...</div>;
  }

  const comparison = metrics?.manual_comparison;

  return (
    <div className="space-y-6">
      <div className="flex items-start justify-between">
        <div>
          <h3 className="font-semibold text-gray-900 mb-1">Productivity &amp; Measurement Dashboard</h3>
          <p className="text-sm text-gray-500">
            Real execution timings, test outcome deltas, and comparative productivity benchmarks against manual triage.
          </p>
        </div>
        <button
          type="button"
          onClick={() => setShowManualForm(!showManualForm)}
          className="flex items-center gap-1.5 text-xs bg-white border border-gray-300 hover:border-gray-400 text-gray-700 px-3 py-1.5 rounded-md font-medium transition-colors"
        >
          <PlusCircle size={14} className="text-blue-600" />
          {comparison ? 'Update Manual Baseline' : 'Record Manual Baseline'}
        </button>
      </div>

      {recordSuccess && (
        <div className="bg-emerald-50 border border-emerald-200 text-emerald-800 text-xs px-3.5 py-2.5 rounded-md flex items-center gap-2">
          <Check size={16} />
          Manual developer baseline benchmark saved! Metrics comparison updated.
        </div>
      )}

      {/* Manual Baseline Form */}
      {showManualForm && (
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 space-y-3">
          <div className="text-xs font-semibold text-blue-900 uppercase tracking-wide">
            Record Developer Manual Triage Benchmark
          </div>
          <p className="text-xs text-blue-700">
            Enter the active engineering minutes required for manual root-cause analysis, code correction, and verification of this contract defect.
          </p>
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-2">
              <label htmlFor="manual-mins" className="text-xs font-medium text-gray-700">
                Active Minutes:
              </label>
              <input
                id="manual-mins"
                type="number"
                min="5"
                max="300"
                value={manualMinutes}
                onChange={(e) => setManualMinutes(Number(e.target.value))}
                className="w-24 text-xs border border-gray-300 rounded px-2.5 py-1.5 bg-white font-mono"
              />
            </div>
            <button
              type="button"
              onClick={() => recordMutation.mutate(manualMinutes)}
              disabled={recordMutation.isPending}
              className="text-xs bg-blue-600 hover:bg-blue-700 text-white font-medium px-3.5 py-1.5 rounded disabled:opacity-50"
            >
              {recordMutation.isPending ? 'Saving...' : 'Save Benchmark'}
            </button>
            <button
              type="button"
              onClick={() => setShowManualForm(false)}
              className="text-xs text-gray-500 hover:text-gray-700"
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* Key Metric KPI Cards */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="bg-white border border-gray-200 rounded-lg p-4 shadow-sm">
          <div className="flex items-center gap-2 text-xs font-medium text-gray-500 mb-1">
            <AlertOctagon size={16} className="text-red-500" />
            Baseline Defects
          </div>
          <div className="text-2xl font-bold text-gray-900">
            {metrics?.baseline_tests_failed ?? 0}
          </div>
          <div className="text-[11px] text-gray-400 mt-1">Failing tests in broken baseline</div>
        </div>

        <div className="bg-white border border-gray-200 rounded-lg p-4 shadow-sm">
          <div className="flex items-center gap-2 text-xs font-medium text-gray-500 mb-1">
            <CheckCircle size={16} className="text-emerald-500" />
            Verified Tests
          </div>
          <div className="text-2xl font-bold text-emerald-700">
            {metrics?.verification_tests_passed ?? 0}
          </div>
          <div className="text-[11px] text-gray-400 mt-1">Passing tests post-fix</div>
        </div>

        <div className="bg-white border border-gray-200 rounded-lg p-4 shadow-sm">
          <div className="flex items-center gap-2 text-xs font-medium text-gray-500 mb-1">
            <FileCode size={16} className="text-blue-500" />
            Modified Files
          </div>
          <div className="text-2xl font-bold text-gray-900">
            {metrics?.modified_files ?? 0}
          </div>
          <div className="text-[11px] text-gray-400 mt-1">Files changed in workspace</div>
        </div>

        <div className="bg-white border border-gray-200 rounded-lg p-4 shadow-sm">
          <div className="flex items-center gap-2 text-xs font-medium text-gray-500 mb-1">
            <Clock size={16} className="text-purple-500" />
            Total Investigation Time
          </div>
          <div className="text-2xl font-bold text-gray-900">
            {metrics?.total_duration_ms ? formatDuration(metrics.total_duration_ms) : '—'}
          </div>
          <div className="text-[11px] text-gray-400 mt-1">End-to-end duration</div>
        </div>
      </div>

      {/* Comparative Benchmark Section */}
      <div className="bg-white border border-gray-200 rounded-lg p-5 shadow-sm space-y-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <TrendingUp size={18} className="text-blue-600" />
            <h4 className="font-semibold text-gray-900 text-sm">
              Productivity Acceleration Benchmark
            </h4>
          </div>
          {comparison && (
            <span className="text-xs bg-emerald-50 text-emerald-800 border border-emerald-200 px-2 py-0.5 rounded-full font-semibold">
              {comparison.percentage_time_reduction}% Faster
            </span>
          )}
        </div>

        {comparison ? (
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div className="border border-gray-100 bg-gray-50 rounded-lg p-3.5 text-center">
              <div className="text-xs font-semibold text-gray-500 uppercase tracking-wide">
                Manual Active Time
              </div>
              <div className="text-2xl font-bold text-gray-800 mt-1">
                {comparison.manual_active_minutes} <span className="text-xs font-normal">min</span>
              </div>
              <div className="text-[11px] text-gray-400 mt-0.5">Developer triage baseline</div>
            </div>

            <div className="border border-blue-100 bg-blue-50 rounded-lg p-3.5 text-center">
              <div className="text-xs font-semibold text-blue-700 uppercase tracking-wide">
                Bob AI Active Time
              </div>
              <div className="text-2xl font-bold text-blue-800 mt-1">
                {comparison.bob_active_minutes} <span className="text-xs font-normal">min</span>
              </div>
              <div className="text-[11px] text-blue-600 mt-0.5">Automated investigation &amp; patch</div>
            </div>

            <div className="border border-emerald-100 bg-emerald-50 rounded-lg p-3.5 text-center">
              <div className="text-xs font-semibold text-emerald-700 uppercase tracking-wide">
                Time Saved
              </div>
              <div className="text-2xl font-bold text-emerald-800 mt-1">
                {comparison.time_saved_minutes} <span className="text-xs font-normal">min</span>
              </div>
              <div className="text-[11px] text-emerald-600 mt-0.5">Engineering hours reclaimed</div>
            </div>

            <div className="border border-purple-100 bg-purple-50 rounded-lg p-3.5 text-center">
              <div className="text-xs font-semibold text-purple-700 uppercase tracking-wide">
                Time Reduction
              </div>
              <div className="text-2xl font-bold text-purple-800 mt-1">
                {comparison.percentage_time_reduction}%
              </div>
              <div className="text-[11px] text-purple-600 mt-0.5">Efficiency acceleration</div>
            </div>
          </div>
        ) : (
          <div className="bg-gray-50 border border-dashed border-gray-200 rounded-lg p-6 text-center">
            <p className="text-xs text-gray-600 font-medium">
              No developer manual baseline recorded yet for comparison.
            </p>
            <p className="text-[11px] text-gray-400 mt-1">
              Click &quot;Record Manual Baseline&quot; above to log the developer baseline for this scenario and generate time savings metrics.
            </p>
          </div>
        )}

        <div className="border-t border-gray-100 pt-3 text-xs text-gray-500 grid grid-cols-2 gap-4">
          <div>
            <strong>Baseline Duration:</strong> {formatDuration(metrics?.baseline_duration_ms)}
          </div>
          <div>
            <strong>Verification Duration:</strong> {formatDuration(metrics?.verification_duration_ms)}
          </div>
        </div>
      </div>
    </div>
  );
}
