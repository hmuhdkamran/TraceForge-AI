import { useQuery } from '@tanstack/react-query';
import { Link } from 'react-router-dom';
import { listInvestigations } from '../services/api';
import type { Investigation } from '../types';
import { formatDatetime } from '../utils/status';
import { StatusBadge } from '../components/shared/StatusBadge';

export function EvidencePage() {
  const { data, isLoading } = useQuery({
    queryKey: ['investigations', 'all'],
    queryFn: () => listInvestigations(100, 0),
  });

  const completed = (data?.investigations ?? []).filter(
    (i) => ['VERIFIED', 'COMPLETED'].includes(i.status)
  );

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">Evidence</h1>
        <p className="text-gray-500 text-sm mt-1">Browse contract evidence graphs from completed investigations</p>
      </div>

      {isLoading && <div className="text-gray-400">Loading...</div>}

      {!isLoading && completed.length === 0 && (
        <div className="bg-white rounded-lg border border-gray-200 p-12 text-center">
          <p className="text-gray-500">No completed investigations with evidence yet.</p>
          <p className="text-gray-400 text-sm mt-1">
            Complete an investigation to see its evidence graph here.
          </p>
        </div>
      )}

      <div className="grid gap-4">
        {completed.map((inv) => (
          <Link key={inv.id} to={`/investigations/${inv.id}`} className="block">
            <div className="bg-white rounded-lg border border-gray-200 p-4 hover:border-blue-300 hover:shadow-sm transition-all">
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium text-gray-900">{inv.title}</div>
                  <div className="text-xs text-gray-400 mt-1">
                    {inv.project_id} · {formatDatetime(inv.completed_at || inv.updated_at)}
                  </div>
                </div>
                <StatusBadge status={inv.status} />
              </div>
            </div>
          </Link>
        ))}
      </div>
    </div>
  );
}
