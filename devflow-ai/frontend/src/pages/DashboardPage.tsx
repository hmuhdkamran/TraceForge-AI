import { useQuery } from '@tanstack/react-query';
import { Link } from 'react-router-dom';
import { Activity, CheckCircle, Clock, AlertTriangle, PlusCircle } from 'lucide-react';
import { listInvestigations } from '../services/api';
import { StatusBadge } from '../components/shared/StatusBadge';
import { formatDatetime } from '../utils/status';

export function DashboardPage() {
  const { data, isLoading, error } = useQuery({
    queryKey: ['investigations', 'list'],
    queryFn: () => listInvestigations(20, 0),
    refetchInterval: 15_000,
  });

  const investigations = data?.investigations ?? [];
  const total = data?.total ?? 0;

  const inProgress = investigations.filter((i) =>
    ['BASELINE_RUNNING', 'AWAITING_BOB_INVESTIGATION', 'INVESTIGATION_IMPORTED', 'VERIFICATION_RUNNING', 'IMPLEMENTATION_IN_PROGRESS'].includes(i.status)
  ).length;

  const awaitingApproval = investigations.filter((i) =>
    ['AWAITING_APPROVAL', 'APPROVED'].includes(i.status)
  ).length;

  const verified = investigations.filter((i) =>
    ['VERIFIED', 'COMPLETED'].includes(i.status)
  ).length;

  const statCards = [
    { label: 'Total Investigations', value: total, icon: Activity, color: 'text-blue-600 bg-blue-50' },
    { label: 'In Progress', value: inProgress, icon: Clock, color: 'text-amber-600 bg-amber-50' },
    { label: 'Awaiting Approval', value: awaitingApproval, icon: AlertTriangle, color: 'text-orange-600 bg-orange-50' },
    { label: 'Verified', value: verified, icon: CheckCircle, color: 'text-emerald-600 bg-emerald-50' },
  ];

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Dashboard</h1>
          <p className="text-gray-500 text-sm mt-1">Track API contract investigations from bug to verified fix</p>
        </div>
        <Link
          to="/investigations/new"
          className="inline-flex items-center gap-2 bg-blue-600 text-white px-4 py-2 rounded-md text-sm font-medium hover:bg-blue-700"
        >
          <PlusCircle size={16} />
          New Investigation
        </Link>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        {statCards.map(({ label, value, icon: Icon, color }) => (
          <div key={label} className="bg-white rounded-lg border border-gray-200 p-4">
            <div className="flex items-center gap-3">
              <div className={`p-2 rounded-md ${color}`}>
                <Icon size={20} />
              </div>
              <div>
                <div className="text-2xl font-bold text-gray-900">
                  {isLoading ? '—' : value}
                </div>
                <div className="text-xs text-gray-500">{label}</div>
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Investigations table */}
      <div className="bg-white rounded-lg border border-gray-200">
        <div className="px-6 py-4 border-b border-gray-100">
          <h2 className="font-semibold text-gray-900">Recent Investigations</h2>
        </div>

        {isLoading && (
          <div className="p-8 text-center text-gray-400">Loading...</div>
        )}

        {error && (
          <div className="p-8 text-center text-red-500">
            Could not load investigations. Is the backend running?
          </div>
        )}

        {!isLoading && !error && investigations.length === 0 && (
          <div className="p-12 text-center">
            <Activity size={48} className="mx-auto text-gray-300 mb-3" />
            <p className="text-gray-500 font-medium">No investigations yet</p>
            <p className="text-gray-400 text-sm mt-1">Create your first investigation to get started</p>
            <Link
              to="/investigations/new"
              className="inline-flex items-center gap-2 mt-4 bg-blue-600 text-white px-4 py-2 rounded-md text-sm font-medium hover:bg-blue-700"
            >
              <PlusCircle size={16} />
              New Investigation
            </Link>
          </div>
        )}

        {investigations.length > 0 && (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-gray-100 bg-gray-50">
                  <th className="text-left px-6 py-3 font-medium text-gray-600">Title</th>
                  <th className="text-left px-6 py-3 font-medium text-gray-600">Project</th>
                  <th className="text-left px-6 py-3 font-medium text-gray-600">Status</th>
                  <th className="text-left px-6 py-3 font-medium text-gray-600">Created</th>
                  <th className="text-left px-6 py-3 font-medium text-gray-600"></th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-50">
                {investigations.map((inv) => (
                  <tr key={inv.id} className="hover:bg-gray-50">
                    <td className="px-6 py-3 font-medium text-gray-900">{inv.title}</td>
                    <td className="px-6 py-3 text-gray-500 font-mono text-xs">{inv.project_id}</td>
                    <td className="px-6 py-3">
                      <StatusBadge status={inv.status} />
                    </td>
                    <td className="px-6 py-3 text-gray-400 text-xs">{formatDatetime(inv.created_at)}</td>
                    <td className="px-6 py-3">
                      <Link
                        to={`/investigations/${inv.id}`}
                        className="text-blue-600 hover:underline text-xs"
                      >
                        View →
                      </Link>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
