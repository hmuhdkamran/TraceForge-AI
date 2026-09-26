import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getProjects, getScenarios, createInvestigation } from '../services/api';
import { createInvestigationSchema } from '../schemas';

export function NewInvestigationPage() {
  const navigate = useNavigate();
  const qc = useQueryClient();

  const { data: projects = [] } = useQuery({
    queryKey: ['projects'],
    queryFn: getProjects,
  });

  const [projectId, setProjectId] = useState('uploadlab');
  const [scenarioId, setScenarioId] = useState('');
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [expectedBehavior, setExpectedBehavior] = useState('');
  const [observedBehavior, setObservedBehavior] = useState('');
  const [reproSteps, setReproSteps] = useState('');
  const [errors, setErrors] = useState<Record<string, string>>({});

  const { data: scenarios = [] } = useQuery({
    queryKey: ['scenarios', projectId],
    queryFn: () => getScenarios(projectId),
    enabled: !!projectId,
  });

  // Pre-fill scenario on load
  useEffect(() => {
    if (scenarios.length > 0 && !scenarioId) {
      const s = scenarios[0];
      setScenarioId(s.id);
      setTitle(s.name);
      setDescription(s.bug_report);
    }
  }, [scenarios, scenarioId]);

  const mutation = useMutation({
    mutationFn: createInvestigation,
    onSuccess: (inv) => {
      qc.invalidateQueries({ queryKey: ['investigations'] });
      navigate(`/investigations/${inv.id}`);
    },
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    const result = createInvestigationSchema.safeParse({
      title,
      description,
      project_id: projectId,
      scenario_id: scenarioId,
      expected_behavior: expectedBehavior,
      observed_behavior: observedBehavior,
      reproduction_steps: reproSteps,
    });

    if (!result.success) {
      const fieldErrors: Record<string, string> = {};
      for (const issue of result.error.issues) {
        const field = issue.path[0] as string;
        fieldErrors[field] = issue.message;
      }
      setErrors(fieldErrors);
      return;
    }

    setErrors({});
    mutation.mutate(result.data);
  };

  const selectedScenario = scenarios.find((s) => s.id === scenarioId);

  return (
    <div className="max-w-2xl mx-auto space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">New Investigation</h1>
        <p className="text-gray-500 text-sm mt-1">Create an API contract investigation</p>
      </div>

      <form onSubmit={handleSubmit} className="space-y-5 bg-white rounded-lg border border-gray-200 p-6">
        {/* Project */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Project</label>
          <select
            value={projectId}
            onChange={(e) => setProjectId(e.target.value)}
            className="w-full border border-gray-300 rounded-md px-3 py-2 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          >
            {projects.map((p) => (
              <option key={p.id} value={p.id}>{p.name}</option>
            ))}
          </select>
        </div>

        {/* Scenario */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Scenario</label>
          <select
            value={scenarioId}
            onChange={(e) => {
              setScenarioId(e.target.value);
              const s = scenarios.find((sc) => sc.id === e.target.value);
              if (s) {
                setTitle(s.name);
                setDescription(s.bug_report);
              }
            }}
            className="w-full border border-gray-300 rounded-md px-3 py-2 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="">— select scenario —</option>
            {scenarios.map((s) => (
              <option key={s.id} value={s.id}>{s.name}</option>
            ))}
          </select>
          {errors.scenario_id && <p className="text-red-500 text-xs mt-1">{errors.scenario_id}</p>}
        </div>

        {/* Title */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Bug Title *</label>
          <input
            type="text"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="e.g. Multi-file upload only returns one file"
            className="w-full border border-gray-300 rounded-md px-3 py-2 text-sm focus:ring-2 focus:ring-blue-500"
          />
          {errors.title && <p className="text-red-500 text-xs mt-1">{errors.title}</p>}
        </div>

        {/* Description */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Bug Description *</label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            rows={4}
            placeholder="Describe the issue in detail..."
            className="w-full border border-gray-300 rounded-md px-3 py-2 text-sm focus:ring-2 focus:ring-blue-500"
          />
          {errors.description && <p className="text-red-500 text-xs mt-1">{errors.description}</p>}
        </div>

        {/* Expected */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Expected Behavior</label>
          <textarea
            value={expectedBehavior}
            onChange={(e) => setExpectedBehavior(e.target.value)}
            rows={2}
            placeholder="What should happen?"
            className="w-full border border-gray-300 rounded-md px-3 py-2 text-sm focus:ring-2 focus:ring-blue-500"
          />
        </div>

        {/* Observed */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Observed Behavior</label>
          <textarea
            value={observedBehavior}
            onChange={(e) => setObservedBehavior(e.target.value)}
            rows={2}
            placeholder="What actually happens?"
            className="w-full border border-gray-300 rounded-md px-3 py-2 text-sm focus:ring-2 focus:ring-blue-500"
          />
        </div>

        {/* Repro steps */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Reproduction Steps</label>
          <textarea
            value={reproSteps}
            onChange={(e) => setReproSteps(e.target.value)}
            rows={3}
            placeholder="Steps to reproduce..."
            className="w-full border border-gray-300 rounded-md px-3 py-2 text-sm focus:ring-2 focus:ring-blue-500"
          />
        </div>

        {/* Doc preview */}
        {selectedScenario && (
          <div className="bg-blue-50 border border-blue-200 rounded-md p-3 text-xs text-blue-800">
            <div className="font-medium mb-1">Scenario: {selectedScenario.name}</div>
            <div className="text-blue-600">{selectedScenario.description}</div>
          </div>
        )}

        {mutation.error && (
          <div className="bg-red-50 border border-red-200 rounded-md p-3 text-sm text-red-700">
            {String(mutation.error)}
          </div>
        )}

        <div className="flex gap-3 pt-2">
          <button
            type="submit"
            disabled={mutation.isPending}
            className="bg-blue-600 text-white px-6 py-2 rounded-md text-sm font-medium hover:bg-blue-700 disabled:opacity-50"
          >
            {mutation.isPending ? 'Creating...' : 'Submit Investigation'}
          </button>
          <button
            type="button"
            onClick={() => navigate('/')}
            className="px-4 py-2 rounded-md text-sm text-gray-600 hover:bg-gray-100"
          >
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}
