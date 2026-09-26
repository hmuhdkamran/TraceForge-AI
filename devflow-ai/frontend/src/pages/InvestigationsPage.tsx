import { useQuery } from '@tanstack/react-query';
import { Link } from 'react-router-dom';
import { PlusCircle } from 'lucide-react';
import { listInvestigations } from '../services/api';
import { StatusBadge } from '../components/shared/StatusBadge';
import { formatDatetime } from '../utils/status';

export function InvestigationsPage() {
  const { data, isLoading, error } = useQuery({
    queryKey: ['investigations', 'list'],
    queryFn: () => listInvestigations(50, 0),
    refetchInterval: 10_000,
  });

  const investigations = data?.investigations ?? [];

  return (
    <div className="space-y-5">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-gray-900">Investigations</h1>
        <Link
          to="/investigations/new"
          className="inline-flex items-center gap-2 bg-blue-600 text-white px-4 py-2 rounded-md text-sm font-medium hover:bg-blue-700"
        >
          <PlusCircle size={16} />
          New
        </Link>
      </div>

      <div className="bg-white rounded-lg border border-gray-200">
        {isLoading && <div className="p-8 text-center text-gray-400">Loading...</div>}
        {error && <div className="p-8 text-center text-red-500">Failed to load investigations.</div>}
        {!isLoading && investigations.length === 0 && (
          <div className="p-12 text-center text-gray-400">
            No investigations yet.{' '}
            <Link to="/investigations/new" className="text-blue-600 hover:underline">Create one</Link>
          </div>
        )}
        {investigations.length > 0 && (
          <table className="w-full text-sm">
            <thead>
              <tr className="bg-gray-50 border-b border-gray-100">
                <th className="text-left px-6 py-3 font-medium text-gray-600">Title</th>
                <th className="text-left px-6 py-3 font-medium text-gray-600">Project</th>
                <th className="text-left px-6 py-3 font-medium text-gray-600">Status</th>
                <th className="text-left px-6 py-3 font-medium text-gray-600">Created</th>
                <th className="text-left px-6 py-3 font-medium text-gray-600">Updated</th>
                <th className="px-6 py-3"></th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-50">
              {investigations.map((inv) => (
                <tr key={inv.id} className="hover:bg-gray-50">
                  <td className="px-6 py-3 font-medium text-gray-900">{inv.title}</td>
                  <td className="px-6 py-3 text-gray-500 font-mono text-xs">{inv.project_id}</td>
                  <td className="px-6 py-3"><StatusBadge status={inv.status} /></td>
                  <td className="px-6 py-3 text-xs text-gray-400">{formatDatetime(inv.created_at)}</td>
                  <td className="px-6 py-3 text-xs text-gray-400">{formatDatetime(inv.updated_at)}</td>
                  <td className="px-6 py-3">
                    <Link to={`/investigations/${inv.id}`} className="text-blue-600 hover:underline text-xs">View →</Link>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}
