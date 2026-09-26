import { useState } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import { Copy, RefreshCw, Check, AlertCircle } from 'lucide-react';
import { getBobInvestigationPrompt, syncArtifacts } from '../../services/api';
import type { Investigation } from '../../types';
import { formatDatetime } from '../../utils/status';

interface Props {
  investigation: Investigation;
  onSync: () => void;
}

export function BobWorkspacePanel({ investigation, onSync }: Props) {
  const [copied, setCopied] = useState(false);
  const [syncResult, setSyncResult] = useState<{ artifacts_imported: number; findings_created: number; errors: string[] } | null>(null);

  const { data: promptData } = useQuery({
    queryKey: ['bob-prompt', investigation.id],
    queryFn: () => getBobInvestigationPrompt(investigation.id),
  });

  const syncMutation = useMutation({
    mutationFn: () => syncArtifacts(investigation.id),
    onSuccess: (result) => {
      setSyncResult(result);
      onSync();
    },
  });

  const handleCopy = async () => {
    if (promptData?.prompt) {
      await navigator.clipboard.writeText(promptData.prompt);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div className="space-y-5">
      <div>
        <h3 className="font-semibold text-gray-900 mb-1">IBM Bob Workspace</h3>
        <p className="text-sm text-gray-500">
          Open the workspace in IBM Bob, complete the investigation, then sync the results back.
        </p>
      </div>

      {/* Workspace path */}
      {promptData && (
        <div>
          <div className="text-xs font-medium text-gray-600 mb-1">Workspace Path</div>
          <code className="block bg-gray-50 border border-gray-200 rounded px-3 py-2 text-xs font-mono text-gray-700 break-all">
            {promptData.workspace_path}
          </code>
        </div>
      )}

      {/* Required artifacts */}
      {promptData && (
        <div>
          <div className="text-xs font-medium text-gray-600 mb-2">Required Artifacts</div>
          <ul className="space-y-1">
            {promptData.required_artifacts.map((a) => (
              <li key={a} className="flex items-center gap-2 text-xs">
                <span className="text-amber-500">○</span>
                <code className="font-mono text-gray-600">{a}</code>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Investigation prompt */}
      <div>
        <div className="flex items-center justify-between mb-2">
          <div className="text-xs font-medium text-gray-600">Investigation Prompt</div>
          <button
            onClick={handleCopy}
            className="flex items-center gap-1.5 text-xs text-blue-600 hover:text-blue-800 px-2 py-1 rounded hover:bg-blue-50"
          >
            {copied ? <Check size={12} /> : <Copy size={12} />}
            {copied ? 'Copied!' : 'Copy Prompt'}
          </button>
        </div>
        <pre className="bg-gray-50 border border-gray-200 rounded p-3 text-xs text-gray-700 overflow-y-auto max-h-72 whitespace-pre-wrap font-mono">
          {promptData?.prompt ?? 'Loading prompt...'}
        </pre>
      </div>

      {/* Sync button */}
      <div>
        <button
          onClick={() => syncMutation.mutate()}
          disabled={syncMutation.isPending}
          className="flex items-center gap-2 bg-teal-600 text-white px-4 py-2 rounded-md text-sm font-medium hover:bg-teal-700 disabled:opacity-50"
        >
          <RefreshCw size={14} className={syncMutation.isPending ? 'animate-spin' : ''} />
          {syncMutation.isPending ? 'Syncing...' : 'Sync Results'}
        </button>

        {syncResult && (
          <div className={`mt-3 p-3 rounded-md border text-sm ${syncResult.errors.length === 0 ? 'bg-emerald-50 border-emerald-200 text-emerald-700' : 'bg-amber-50 border-amber-200 text-amber-700'}`}>
            <div className="font-medium">
              {syncResult.artifacts_imported} artifact{syncResult.artifacts_imported !== 1 ? 's' : ''} imported,{' '}
              {syncResult.findings_created} finding{syncResult.findings_created !== 1 ? 's' : ''} created
            </div>
            {syncResult.errors.length > 0 && (
              <ul className="mt-2 space-y-1">
                {syncResult.errors.map((e, i) => (
                  <li key={i} className="flex items-center gap-1.5 text-xs">
                    <AlertCircle size={12} />
                    {e}
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}

        {syncMutation.error && (
          <div className="mt-3 text-red-600 text-sm">
            Sync failed: {String(syncMutation.error)}
          </div>
        )}
      </div>

      {['AWAITING_BOB_INVESTIGATION', 'CREATED', 'WORKSPACE_READY', 'BASELINE_CAPTURED'].includes(investigation.status) && !syncResult && (
        <div className="flex items-center gap-2 text-sm text-amber-700 bg-amber-50 border border-amber-200 rounded-md p-3">
          <AlertCircle size={16} />
          <span>Waiting for Bob investigation artifacts. Run the investigation in IBM Bob, then click <strong>Sync Results</strong>.</span>
        </div>
      )}
    </div>
  );
}
