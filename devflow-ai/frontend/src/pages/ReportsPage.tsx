import { useQuery } from '@tanstack/react-query';
import { Link } from 'react-router-dom';
import { FileText, Download } from 'lucide-react';
import { listInvestigations } from '../services/api';
import { StatusBadge } from '../components/shared/StatusBadge';
import { formatDatetime } from '../utils/status';

export function ReportsPage() {
  const { data, isLoading } = useQuery({
    queryKey: ['investigations', 'reports'],
    queryFn: () => listInvestigations(100, 0),
  });

  const completed = (data?.investigations ?? []).filter(
    (i) => ['VERIFIED', 'COMPLETED', 'FAILED', 'VERIFICATION_FAILED'].includes(i.status)
  );

  const apiBase = import.meta.env.VITE_API_URL || '/api/v1';

  return (
    <div className="space-y-5">
      <h1 className="text-2xl font-bold text-gray-900">Reports</h1>

      {isLoading && <div className="text-gray-400">Loading...</div>}

      {!isLoading && completed.length === 0 && (
        <div className="bg-white rounded-lg border border-gray-200 p-12 text-center">
          <FileText size={48} className="mx-auto text-gray-300 mb-3" />
          <p className="text-gray-500">No investigation reports yet.</p>
        </div>
      )}

      <div className="space-y-3">
        {completed.map((inv) => (
          <div key={inv.id} className="bg-white rounded-lg border border-gray-200 p-4">
            <div className="flex items-start justify-between gap-4">
              <div>
                <div className="font-medium text-gray-900">{inv.title}</div>
                <div className="text-xs text-gray-400 mt-1">
                  {inv.project_id} · {formatDatetime(inv.updated_at)}
                </div>
              </div>
              <div className="flex items-center gap-2">
                <StatusBadge status={inv.status} />
                <Link
                  to={`/investigations/${inv.id}`}
                  className="text-xs text-blue-600 hover:underline"
                >
                  View Details
                </Link>
                <a
                  href={`${apiBase}/investigations/${inv.id}/report.html`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center gap-1 text-xs text-gray-600 hover:text-gray-900 px-2 py-1 rounded border border-gray-200 hover:border-gray-300"
                >
                  <Download size={12} />
                  HTML
                </a>
                <a
                  href={`${apiBase}/investigations/${inv.id}/report.json`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center gap-1 text-xs text-gray-600 hover:text-gray-900 px-2 py-1 rounded border border-gray-200 hover:border-gray-300"
                >
                  <Download size={12} />
                  JSON
                </a>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
