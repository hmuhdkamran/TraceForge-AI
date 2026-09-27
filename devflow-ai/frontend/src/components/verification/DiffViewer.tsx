import { useState } from 'react';
import { FileCode, ChevronDown, ChevronRight, Check } from 'lucide-react';
import type { DiffResponse } from '../../types';

interface Props {
  diffData: DiffResponse;
}

export function DiffViewer({ diffData }: Props) {
  const [expandedFiles, setExpandedFiles] = useState<Record<string, boolean>>({});

  const fileDiffs = diffData.file_diffs || [];
  const rawDiff = diffData.diff || '';

  const toggleFile = (path: string) => {
    setExpandedFiles((prev) => ({
      ...prev,
      [path]: prev[path] === undefined ? false : !prev[path],
    }));
  };

  const changedFilesList: Array<{ relative_path: string; change_type?: string; rationale?: string }> =
    (diffData.changed_files as any)?.files || [];

  return (
    <div className="space-y-4">
      {/* Implementation Summary */}
      {diffData.implementation_summary && (
        <div>
          <div className="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-1.5">
            Implementation Summary
          </div>
          <div className="bg-gray-50 border border-gray-200 rounded-lg p-3 text-xs text-gray-700 whitespace-pre-wrap leading-relaxed">
            {diffData.implementation_summary}
          </div>
        </div>
      )}

      {/* Changed Files Overview */}
      {changedFilesList.length > 0 && (
        <div>
          <div className="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-1.5">
            Modified Files ({changedFilesList.length})
          </div>
          <div className="grid gap-2">
            {changedFilesList.map((file, idx) => (
              <div
                key={idx}
                className="flex items-start gap-2.5 bg-white border border-gray-200 rounded-md p-2.5 text-xs"
              >
                <FileCode size={16} className="text-blue-600 shrink-0 mt-0.5" />
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="font-mono font-medium text-gray-900 truncate">
                      {file.relative_path}
                    </span>
                    <span className="px-1.5 py-0.5 rounded text-[10px] font-semibold uppercase bg-blue-50 text-blue-700 border border-blue-200">
                      {file.change_type || 'modify'}
                    </span>
                  </div>
                  {file.rationale && (
                    <div className="text-gray-500 mt-1 text-[11px] leading-normal">
                      {file.rationale}
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Code Diffs */}
      <div>
        <div className="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-1.5">
          Actual Code Diff
        </div>

        {fileDiffs.length > 0 ? (
          <div className="space-y-3">
            {fileDiffs.map((fd) => {
              const isCollapsed = expandedFiles[fd.path] === false;
              return (
                <div key={fd.path} className="border border-gray-200 rounded-lg overflow-hidden bg-white shadow-sm">
                  <button
                    type="button"
                    onClick={() => toggleFile(fd.path)}
                    className="w-full flex items-center justify-between px-3.5 py-2.5 bg-gray-50 border-b border-gray-200 text-left hover:bg-gray-100 transition-colors"
                  >
                    <div className="flex items-center gap-2 font-mono text-xs font-medium text-gray-800">
                      {isCollapsed ? <ChevronRight size={14} /> : <ChevronDown size={14} />}
                      <FileCode size={14} className="text-blue-600" />
                      <span>{fd.path}</span>
                    </div>
                    <span className="text-[11px] text-gray-500 font-sans">
                      {isCollapsed ? 'Show diff' : 'Hide diff'}
                    </span>
                  </button>

                  {!isCollapsed && (
                    <div className="font-mono text-xs overflow-x-auto divide-y divide-gray-100 max-h-96">
                      {renderDiffLines(fd.diff)}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        ) : rawDiff ? (
          <div className="border border-gray-200 rounded-lg overflow-hidden bg-white shadow-sm">
            <div className="font-mono text-xs overflow-x-auto divide-y divide-gray-100 max-h-96">
              {renderDiffLines(rawDiff)}
            </div>
          </div>
        ) : (
          <div className="bg-gray-50 border border-gray-200 rounded p-4 text-center text-xs text-gray-400">
            No code changes recorded yet.
          </div>
        )}
      </div>
    </div>
  );
}

function renderDiffLines(diffText: string) {
  const lines = diffText.split('\n');
  return lines.map((line, idx) => {
    let lineClass = 'text-gray-700 bg-white hover:bg-gray-50';
    if (line.startsWith('+') && !line.startsWith('+++')) {
      lineClass = 'bg-emerald-50 text-emerald-800 font-medium';
    } else if (line.startsWith('-') && !line.startsWith('---')) {
      lineClass = 'bg-red-50 text-red-800 font-medium';
    } else if (line.startsWith('@@')) {
      lineClass = 'bg-blue-50 text-blue-700 font-semibold';
    } else if (line.startsWith('---') || line.startsWith('+++')) {
      lineClass = 'bg-gray-100 text-gray-600 font-semibold';
    }

    return (
      <div key={idx} className={`px-3 py-0.5 flex items-start gap-3 ${lineClass}`}>
        <span className="text-[11px] select-none opacity-40 w-6 text-right shrink-0">
          {idx + 1}
        </span>
        <span className="whitespace-pre-wrap break-all">{line || ' '}</span>
      </div>
    );
  });
}
