import { Cpu, GitBranch, CheckCircle, Users } from 'lucide-react';

export function AboutPage() {
  return (
    <div className="max-w-2xl mx-auto space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">About ContractGuard</h1>
        <p className="text-gray-500 text-sm mt-1">TraceForge AI — From Bug Report to Verified Fix</p>
      </div>

      <div className="bg-white rounded-lg border border-gray-200 p-6 space-y-5">
        <div className="flex items-start gap-4">
          <div className="p-3 bg-blue-50 rounded-lg">
            <Cpu size={24} className="text-blue-600" />
          </div>
          <div>
            <h2 className="font-semibold text-gray-900">What is ContractGuard?</h2>
            <p className="text-sm text-gray-600 mt-1">
              ContractGuard is a developer workflow application that coordinates the complete journey
              from an API contract bug report through investigation, IBM Bob-assisted root cause analysis,
              human approval, verified code correction, and final report generation.
            </p>
          </div>
        </div>

        <div className="flex items-start gap-4">
          <div className="p-3 bg-teal-50 rounded-lg">
            <GitBranch size={24} className="text-teal-600" />
          </div>
          <div>
            <h2 className="font-semibold text-gray-900">Contract Evidence Graph</h2>
            <p className="text-sm text-gray-600 mt-1">
              Every investigation builds a traceable evidence graph connecting documented requirements,
              source code findings, failing tests, root causes, approved corrections, and verified test results.
            </p>
          </div>
        </div>

        <div className="flex items-start gap-4">
          <div className="p-3 bg-emerald-50 rounded-lg">
            <CheckCircle size={24} className="text-emerald-600" />
          </div>
          <div>
            <h2 className="font-semibold text-gray-900">IBM Bob Integration</h2>
            <p className="text-sm text-gray-600 mt-1">
              IBM Bob IDE performs the actual investigation: reading documentation, inspecting source code,
              running subagent investigations in parallel, generating structured diagnosis artifacts, and
              implementing approved corrections. ContractGuard orchestrates the workflow and independently
              verifies the results.
            </p>
          </div>
        </div>

        <div className="flex items-start gap-4">
          <div className="p-3 bg-purple-50 rounded-lg">
            <Users size={24} className="text-purple-600" />
          </div>
          <div>
            <h2 className="font-semibold text-gray-900">Hackathon</h2>
            <p className="text-sm text-gray-600 mt-1">
              Built for the IBM Bob 2.0 Hackathon. Uses Rust + Axum for the backend, React + TypeScript
              for the frontend, SQLite for persistence, and IBM Bob IDE for AI-assisted investigation.
            </p>
          </div>
        </div>
      </div>

      <div className="bg-gray-50 rounded-lg border border-gray-200 p-4 text-xs text-gray-500 space-y-1">
        <div><strong>Backend:</strong> Rust 1.95 · Axum 0.7 · SQLite · SQLx</div>
        <div><strong>Frontend:</strong> React 18 · TypeScript · Vite · Tailwind CSS · TanStack Query</div>
        <div><strong>AI:</strong> IBM Bob 2.0 IDE</div>
        <div><strong>License:</strong> For demonstration purposes — IBM Bob 2.0 Hackathon submission</div>
      </div>
    </div>
  );
}
